use crate::spotify::{SpotifyClient, PlaybackState, Track, Playlist};
use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Gauge, List, ListItem, ListState, Paragraph, Wrap},
    Frame, Terminal,
};
use std::io;
use tokio::time::{Duration, Instant};

#[derive(Debug, PartialEq)]
enum InputMode {
    Normal,
    Search,
    Volume,
}

#[derive(Debug)]
enum AppState {
    Player,
    Search,
    Playlists,
    Favorites,
}

pub struct App {
    spotify_client: SpotifyClient,
    current_playback: Option<PlaybackState>,
    input_mode: InputMode,
    app_state: AppState,
    search_input: String,
    search_results: Vec<Track>,
    search_list_state: ListState,
    volume_input: String,
    error_message: Option<String>,
    success_message: Option<String>,
    last_update: Instant,
    should_quit: bool,
    playlists: Vec<Playlist>,
    playlist_list_state: ListState,
    favorites: Vec<Track>,
    favorites_list_state: ListState,
}

impl App {
    pub fn new(spotify_client: SpotifyClient) -> Self {
        let mut search_list_state = ListState::default();
        search_list_state.select(Some(0));
        
        Self {
            spotify_client,
            current_playback: None,
            input_mode: InputMode::Normal,
            app_state: AppState::Player,
            search_input: String::new(),
            search_results: Vec::new(),
            search_list_state,
            volume_input: String::new(),
            error_message: None,
            success_message: None,
            last_update: Instant::now(),
            should_quit: false,
            playlists: Vec::new(),
            playlist_list_state: ListState::default(),
            favorites: Vec::new(),
            favorites_list_state: ListState::default(),
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // Actualizar estado inicial
        self.update_playback_state().await;

        let mut last_tick = Instant::now();
        let tick_rate = Duration::from_millis(250);

        loop {
            terminal.draw(|f| self.ui(f))?;

            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if crossterm::event::poll(timeout)? {
                if let Event::Key(key) = event::read()? {
                    if self.handle_key_event(key).await? {
                        break;
                    }
                }
            }

            if last_tick.elapsed() >= tick_rate {
                // Actualizar estado de reproducción cada segundo aproximadamente
                if self.last_update.elapsed() >= Duration::from_secs(1) {
                    self.update_playback_state().await;
                    self.last_update = Instant::now();
                }
                last_tick = Instant::now();
            }

            if self.should_quit {
                break;
            }
        }

        // Restore terminal
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        Ok(())
    }

    async fn update_playback_state(&mut self) {
        match self.spotify_client.get_current_playback().await {
            Ok(playback) => {
                self.current_playback = playback;
                self.error_message = None;
            }
            Err(e) => {
                self.error_message = Some(format!("Error al actualizar reproducción: {}", e));
            }
        }
    }

    async fn handle_key_event(&mut self, key: KeyEvent) -> Result<bool> {
        // Clear messages after key press
        self.success_message = None;
        
        match self.input_mode {
            InputMode::Normal => self.handle_normal_key_event(key).await,
            InputMode::Search => self.handle_search_key_event(key).await,
            InputMode::Volume => self.handle_volume_key_event(key).await,
        }
    }

    async fn handle_normal_key_event(&mut self, key: KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Char('q') => return Ok(true),
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return Ok(true),
            
            // Controles de reproducción
            KeyCode::Char(' ') => self.toggle_playback().await,
            KeyCode::Char('n') | KeyCode::Right => self.next_track().await,
            KeyCode::Char('p') | KeyCode::Left => self.previous_track().await,
            KeyCode::Char('s') => self.toggle_shuffle().await,
            KeyCode::Char('r') => self.toggle_repeat().await,
            
            // Navegación entre vistas
            KeyCode::Char('1') => self.app_state = AppState::Player,
            KeyCode::Char('2') => self.app_state = AppState::Search,
            KeyCode::Char('3') => {
                self.app_state = AppState::Playlists;
                self.load_playlists().await;
            }
            KeyCode::Char('4') => {
                self.app_state = AppState::Favorites;
                self.load_favorites().await;
            }
            
            // Búsqueda
            KeyCode::Char('/') => {
                self.input_mode = InputMode::Search;
                self.search_input.clear();
            }
            
            // Control de volumen
            KeyCode::Char('v') => {
                self.input_mode = InputMode::Volume;
                self.volume_input.clear();
            }
            
            // Navegación en resultados de búsqueda
            KeyCode::Up => {
                match self.app_state {
                    AppState::Search => self.previous_search_result(),
                    AppState::Playlists => self.previous_playlist(),
                    AppState::Favorites => self.previous_favorite(),
                    _ => {}
                }
            }
            KeyCode::Down => {
                match self.app_state {
                    AppState::Search => self.next_search_result(),
                    AppState::Playlists => self.next_playlist(),
                    AppState::Favorites => self.next_favorite(),
                    _ => {}
                }
            }
            KeyCode::Enter => {
                match self.app_state {
                    AppState::Search => self.play_selected_track().await,
                    AppState::Playlists => self.play_selected_playlist().await,
                    AppState::Favorites => self.play_selected_favorite().await,
                    _ => {}
                }
            }
            _ => {}
        }
        Ok(false)
    }

    async fn handle_search_key_event(&mut self, key: KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Enter => {
                if !self.search_input.is_empty() {
                    self.perform_search().await;
                }
                self.input_mode = InputMode::Normal;
                self.app_state = AppState::Search;
            }
            KeyCode::Esc => {
                self.input_mode = InputMode::Normal;
            }
            KeyCode::Char(c) => {
                self.search_input.push(c);
            }
            KeyCode::Backspace => {
                self.search_input.pop();
            }
            _ => {}
        }
        Ok(false)
    }

    async fn handle_volume_key_event(&mut self, key: KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Enter => {
                if let Ok(volume) = self.volume_input.parse::<u8>() {
                    if volume <= 100 {
                        self.set_volume(volume).await;
                    } else {
                        self.error_message = Some("El volumen debe estar entre 0 y 100".to_string());
                    }
                } else {
                    self.error_message = Some("Volumen inválido".to_string());
                }
                self.input_mode = InputMode::Normal;
            }
            KeyCode::Esc => {
                self.input_mode = InputMode::Normal;
            }
            KeyCode::Char(c) if c.is_numeric() => {
                if self.volume_input.len() < 3 {
                    self.volume_input.push(c);
                }
            }
            KeyCode::Backspace => {
                self.volume_input.pop();
            }
            _ => {}
        }
        Ok(false)
    }

    async fn toggle_playback(&mut self) {
        if let Some(ref playback) = self.current_playback {
            let result = if playback.is_playing {
                self.spotify_client.pause().await
            } else {
                self.spotify_client.play().await
            };
            
            match result {
                Ok(_) => {
                    self.success_message = Some(if playback.is_playing { "Pausado" } else { "Reproduciendo" }.to_string());
                    // Actualizar estado inmediatamente
                    self.update_playback_state().await;
                }
                Err(e) => self.error_message = Some(format!("Error: {}", e)),
            }
        } else {
            self.error_message = Some("No hay reproducción activa".to_string());
        }
    }

    async fn next_track(&mut self) {
        match self.spotify_client.next_track().await {
            Ok(_) => {
                self.success_message = Some("Siguiente canción".to_string());
                tokio::time::sleep(Duration::from_millis(500)).await;
                self.update_playback_state().await;
            }
            Err(e) => self.error_message = Some(format!("Error: {}", e)),
        }
    }

    async fn previous_track(&mut self) {
        match self.spotify_client.previous_track().await {
            Ok(_) => {
                self.success_message = Some("Canción anterior".to_string());
                tokio::time::sleep(Duration::from_millis(500)).await;
                self.update_playback_state().await;
            }
            Err(e) => self.error_message = Some(format!("Error: {}", e)),
        }
    }

    async fn toggle_shuffle(&mut self) {
        match self.spotify_client.toggle_shuffle().await {
            Ok(_) => {
                self.success_message = Some("Shuffle cambiado".to_string());
                self.update_playback_state().await;
            }
            Err(e) => self.error_message = Some(format!("Error: {}", e)),
        }
    }

    async fn toggle_repeat(&mut self) {
        match self.spotify_client.toggle_repeat().await {
            Ok(_) => {
                self.success_message = Some("Modo repetición cambiado".to_string());
                self.update_playback_state().await;
            }
            Err(e) => self.error_message = Some(format!("Error: {}", e)),
        }
    }

    async fn set_volume(&mut self, volume: u8) {
        match self.spotify_client.set_volume(volume).await {
            Ok(_) => {
                self.success_message = Some(format!("Volumen: {}%", volume));
                self.update_playback_state().await;
            }
            Err(e) => self.error_message = Some(format!("Error: {}", e)),
        }
    }

    async fn perform_search(&mut self) {
        match self.spotify_client.search_tracks(&self.search_input, 20).await {
            Ok(tracks) => {
                self.search_results = tracks;
                self.search_list_state.select(Some(0));
                self.success_message = Some(format!("Encontradas {} canciones", self.search_results.len()));
            }
            Err(e) => self.error_message = Some(format!("Error en búsqueda: {}", e)),
        }
    }

    fn previous_search_result(&mut self) {
        if !self.search_results.is_empty() {
            let i = match self.search_list_state.selected() {
                Some(i) => {
                    if i == 0 {
                        self.search_results.len() - 1
                    } else {
                        i - 1
                    }
                }
                None => 0,
            };
            self.search_list_state.select(Some(i));
        }
    }

    fn next_search_result(&mut self) {
        if !self.search_results.is_empty() {
            let i = match self.search_list_state.selected() {
                Some(i) => {
                    if i >= self.search_results.len() - 1 {
                        0
                    } else {
                        i + 1
                    }
                }
                None => 0,
            };
            self.search_list_state.select(Some(i));
        }
    }

    async fn play_selected_track(&mut self) {
        if let Some(i) = self.search_list_state.selected() {
            if let Some(track) = self.search_results.get(i) {
                let track_uri = format!("spotify:track:{}", track.id);
                match self.spotify_client.play_track(&track_uri).await {
                    Ok(_) => {
                        self.success_message = Some(format!("Reproduciendo: {}", track.name));
                        tokio::time::sleep(Duration::from_millis(500)).await;
                        self.update_playback_state().await;
                    }
                    Err(e) => self.error_message = Some(format!("Error: {}", e)),
                }
            }
        }
    }

    async fn load_playlists(&mut self) {
        match self.spotify_client.get_user_playlists().await {
            Ok(playlists) => {
                self.playlists = playlists;
                self.playlist_list_state.select(Some(0));
                self.success_message = Some(format!("Cargadas {} playlists", self.playlists.len()));
            }
            Err(e) => self.error_message = Some(format!("Error al cargar playlists: {}", e)),
        }
    }

    async fn load_favorites(&mut self) {
        match self.spotify_client.get_saved_tracks().await {
            Ok(tracks) => {
                self.favorites = tracks;
                self.favorites_list_state.select(Some(0));
                self.success_message = Some(format!("Cargadas {} canciones favoritas", self.favorites.len()));
            }
            Err(e) => self.error_message = Some(format!("Error al cargar favoritos: {}", e)),
        }
    }

    fn previous_playlist(&mut self) {
        if !self.playlists.is_empty() {
            let i = match self.playlist_list_state.selected() {
                Some(i) => {
                    if i == 0 {
                        self.playlists.len() - 1
                    } else {
                        i - 1
                    }
                }
                None => 0,
            };
            self.playlist_list_state.select(Some(i));
        }
    }

    fn next_playlist(&mut self) {
        if !self.playlists.is_empty() {
            let i = match self.playlist_list_state.selected() {
                Some(i) => {
                    if i >= self.playlists.len() - 1 {
                        0
                    } else {
                        i + 1
                    }
                }
                None => 0,
            };
            self.playlist_list_state.select(Some(i));
        }
    }

    fn previous_favorite(&mut self) {
        if !self.favorites.is_empty() {
            let i = match self.favorites_list_state.selected() {
                Some(i) => {
                    if i == 0 {
                        self.favorites.len() - 1
                    } else {
                        i - 1
                    }
                }
                None => 0,
            };
            self.favorites_list_state.select(Some(i));
        }
    }

    fn next_favorite(&mut self) {
        if !self.favorites.is_empty() {
            let i = match self.favorites_list_state.selected() {
                Some(i) => {
                    if i >= self.favorites.len() - 1 {
                        0
                    } else {
                        i + 1
                    }
                }
                None => 0,
            };
            self.favorites_list_state.select(Some(i));
        }
    }

    async fn play_selected_playlist(&mut self) {
        if let Some(i) = self.playlist_list_state.selected() {
            if let Some(playlist) = self.playlists.get(i) {
                let playlist_uri = format!("spotify:playlist:{}", playlist.id);
                match self.spotify_client.play_playlist(&playlist_uri).await {
                    Ok(_) => {
                        self.success_message = Some(format!("Reproduciendo playlist: {}", playlist.name));
                        tokio::time::sleep(Duration::from_millis(500)).await;
                        self.update_playback_state().await;
                    }
                    Err(e) => self.error_message = Some(format!("Error: {}", e)),
                }
            }
        }
    }

    async fn play_selected_favorite(&mut self) {
        if let Some(i) = self.favorites_list_state.selected() {
            if let Some(track) = self.favorites.get(i) {
                let track_uri = format!("spotify:track:{}", track.id);
                match self.spotify_client.play_track(&track_uri).await {
                    Ok(_) => {
                        self.success_message = Some(format!("Reproduciendo: {}", track.name));
                        tokio::time::sleep(Duration::from_millis(500)).await;
                        self.update_playback_state().await;
                    }
                    Err(e) => self.error_message = Some(format!("Error: {}", e)),
                }
            }
        }
    }

    fn ui(&mut self, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Min(0),    // Content
                Constraint::Length(3), // Footer
            ])
            .split(f.size());

        self.render_header(f, chunks[0]);
        self.render_content(f, chunks[1]);
        self.render_footer(f, chunks[2]);

        // Render input popups
        if matches!(self.input_mode, InputMode::Search) {
            self.render_search_popup(f);
        } else if matches!(self.input_mode, InputMode::Volume) {
            self.render_volume_popup(f);
        }
    }

    fn render_header(&self, f: &mut Frame, area: Rect) {
        let title = match self.app_state {
            AppState::Player => "🎵 SpotiGod - Reproductor",
            AppState::Search => "🔍 SpotiGod - Búsqueda",
            AppState::Playlists => "📋 SpotiGod - Playlists",
            AppState::Favorites => "🎶 SpotiGod - Favoritos",
        };

        let header = Paragraph::new(title)
            .style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));

        f.render_widget(header, area);
    }

    fn render_content(&mut self, f: &mut Frame, area: Rect) {
        match self.app_state {
            AppState::Player => self.render_player_view(f, area),
            AppState::Search => self.render_search_view(f, area),
            AppState::Playlists => self.render_playlists_view(f, area),
            AppState::Favorites => self.render_favorites_view(f, area),
        }
    }

    fn render_player_view(&self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(8), // Current track info
                Constraint::Length(3), // Progress bar
                Constraint::Length(5), // Controls info
                Constraint::Min(0),    // Status
            ])
            .split(area);

        // Current track info
        if let Some(ref playback) = self.current_playback {
            if let Some(ref track) = playback.item {
                let track_info = vec![
                    Line::from(vec![
                        Span::styled("🎵 ", Style::default().fg(Color::Green)),
                        Span::styled(&track.name, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                    ]),
                    Line::from(vec![
                        Span::styled("👤 ", Style::default().fg(Color::Blue)),
                        Span::styled(
                            track.artists.iter().map(|a| a.name.clone()).collect::<Vec<_>>().join(", "),
                            Style::default().fg(Color::Gray),
                        ),
                    ]),
                    Line::from(vec![
                        Span::styled("💿 ", Style::default().fg(Color::Magenta)),
                        Span::styled(&track.album.name, Style::default().fg(Color::Gray)),
                    ]),
                    Line::from(vec![
                        Span::styled("🎛️  ", Style::default().fg(Color::Yellow)),
                        Span::styled(&playback.device.name, Style::default().fg(Color::Gray)),
                        Span::styled(" | ", Style::default().fg(Color::Gray)),
                        Span::styled(
                            format!("Vol: {}%", playback.device.volume_percent.unwrap_or(0)),
                            Style::default().fg(Color::Gray),
                        ),
                    ]),
                    Line::from(vec![
                        Span::styled("🔀 ", Style::default().fg(if playback.shuffle_state { Color::Green } else { Color::Red })),
                        Span::styled(
                            if playback.shuffle_state { "Shuffle ON" } else { "Shuffle OFF" },
                            Style::default().fg(if playback.shuffle_state { Color::Green } else { Color::Red }),
                        ),
                        Span::styled(" | ", Style::default().fg(Color::Gray)),
                        Span::styled("🔁 ", Style::default().fg(Color::Yellow)),
                        Span::styled(
                            match playback.repeat_state.as_str() {
                                "off" => "Repeat OFF",
                                "context" => "Repeat CONTEXT",
                                "track" => "Repeat TRACK",
                                _ => "Repeat UNKNOWN",
                            },
                            Style::default().fg(Color::Yellow),
                        ),
                    ]),
                ];

                let track_paragraph = Paragraph::new(track_info)
                    .block(Block::default().title("Now Playing").borders(Borders::ALL))
                    .wrap(Wrap { trim: true });

                f.render_widget(track_paragraph, chunks[0]);

                // Progress bar
                if let Some(progress_ms) = playback.progress_ms {
                    let progress = (progress_ms as f64 / track.duration_ms as f64).clamp(0.0, 1.0);
                    let progress_text = format!(
                        "{} / {}",
                        Self::format_duration(progress_ms),
                        Self::format_duration(track.duration_ms)
                    );

                    let progress_bar = Gauge::default()
                        .block(Block::default().title("Progress").borders(Borders::ALL))
                        .gauge_style(Style::default().fg(Color::Green))
                        .percent((progress * 100.0) as u16)
                        .label(progress_text);

                    f.render_widget(progress_bar, chunks[1]);
                } else {
                    let no_progress = Gauge::default()
                        .block(Block::default().title("Progress").borders(Borders::ALL))
                        .gauge_style(Style::default().fg(Color::Gray))
                        .percent(0)
                        .label("-- / --");

                    f.render_widget(no_progress, chunks[1]);
                }
            } else {
                let no_track = Paragraph::new("No hay canción reproduciéndose")
                    .style(Style::default().fg(Color::Yellow))
                    .alignment(Alignment::Center)
                    .block(Block::default().title("Now Playing").borders(Borders::ALL));

                f.render_widget(no_track, chunks[0]);
            }
        } else {
            let no_playback = Paragraph::new("No se detectó reproducción activa\n\nAsegúrate de que Spotify esté abierto\ny reproduciendo música en algún dispositivo")
                .style(Style::default().fg(Color::Red))
                .alignment(Alignment::Center)
                .block(Block::default().title("Now Playing").borders(Borders::ALL));

            f.render_widget(no_playback, chunks[0]);
        }

        // Controls info
        let controls_text = vec![
            Line::from("Controles:"),
            Line::from("SPACE: Play/Pause | ←/p: Anterior | →/n: Siguiente"),
            Line::from("s: Shuffle | r: Repeat | v: Volumen | /: Buscar"),
            Line::from("1: Reproductor | 2: Búsqueda | 3: Playlists | 4: Favoritos | q: Salir"),
        ];

        let controls = Paragraph::new(controls_text)
            .block(Block::default().title("Controles").borders(Borders::ALL))
            .style(Style::default().fg(Color::Cyan));

        f.render_widget(controls, chunks[2]);
    }

    fn render_search_view(&self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Search info
                Constraint::Min(0),    // Results
            ])
            .split(area);

        // Search info
        let search_info = if self.search_results.is_empty() {
            "Presiona '/' para buscar canciones"
        } else {
            "↑/↓: Navegar | Enter: Reproducir | /: Nueva búsqueda"
        };

        let search_paragraph = Paragraph::new(search_info)
            .style(Style::default().fg(Color::Cyan))
            .alignment(Alignment::Center)
            .block(Block::default().title("Búsqueda").borders(Borders::ALL));

        f.render_widget(search_paragraph, chunks[0]);

        // Search results
        if !self.search_results.is_empty() {
            let items: Vec<ListItem> = self
                .search_results
                .iter()
                .enumerate()
                .map(|(i, track)| {
                    let artists = track.artists.iter().map(|a| a.name.clone()).collect::<Vec<_>>().join(", ");
                    let content = Line::from(vec![
                        Span::styled(format!("{:2}. ", i + 1), Style::default().fg(Color::Yellow)),
                        Span::styled(&track.name, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                        Span::styled(" - ", Style::default().fg(Color::Gray)),
                        Span::styled(artists, Style::default().fg(Color::Cyan)),
                        Span::styled(" (", Style::default().fg(Color::Gray)),
                        Span::styled(&track.album.name, Style::default().fg(Color::Magenta)),
                        Span::styled(")", Style::default().fg(Color::Gray)),
                    ]);
                    ListItem::new(content)
                })
                .collect();

            let list = List::new(items)
                .block(Block::default().title("Resultados").borders(Borders::ALL))
                .highlight_style(Style::default().fg(Color::Black).bg(Color::Green))
                .highlight_symbol("► ");

            f.render_stateful_widget(list, chunks[1], &mut self.search_list_state.clone());
        }
    }

    fn render_playlists_view(&self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Título
                Constraint::Min(0),    // Lista de playlists
            ])
            .split(area);

        // Título
        let title = Paragraph::new("Tus Playlists")
            .style(Style::default().fg(Color::Yellow))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));

        f.render_widget(title, chunks[0]);

        // Lista de playlists
        if !self.playlists.is_empty() {
            let items: Vec<ListItem> = self
                .playlists
                .iter()
                .enumerate()
                .map(|(i, playlist)| {
                    let content = Line::from(vec![
                        Span::styled(format!("{:2}. ", i + 1), Style::default().fg(Color::Yellow)),
                        Span::styled(&playlist.name, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                        Span::styled(" - ", Style::default().fg(Color::Gray)),
                        Span::styled(
                            format!("{} canciones", playlist.tracks.total),
                            Style::default().fg(Color::Cyan),
                        ),
                    ]);
                    ListItem::new(content)
                })
                .collect();

            let list = List::new(items)
                .block(Block::default().borders(Borders::ALL))
                .highlight_style(Style::default().fg(Color::Black).bg(Color::Green))
                .highlight_symbol("► ");

            f.render_stateful_widget(list, chunks[1], &mut self.playlist_list_state.clone());
        } else {
            let no_playlists = Paragraph::new("No se encontraron playlists")
                .style(Style::default().fg(Color::Yellow))
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL));

            f.render_widget(no_playlists, chunks[1]);
        }
    }

    fn render_favorites_view(&self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Título
                Constraint::Min(0),    // Lista de favoritos
            ])
            .split(area);

        // Título
        let title = Paragraph::new("Tus Canciones Favoritas")
            .style(Style::default().fg(Color::Yellow))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));

        f.render_widget(title, chunks[0]);

        // Lista de favoritos
        if !self.favorites.is_empty() {
            let items: Vec<ListItem> = self
                .favorites
                .iter()
                .enumerate()
                .map(|(i, track)| {
                    let artists = track.artists.iter().map(|a| a.name.clone()).collect::<Vec<_>>().join(", ");
                    let content = Line::from(vec![
                        Span::styled(format!("{:2}. ", i + 1), Style::default().fg(Color::Yellow)),
                        Span::styled(&track.name, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                        Span::styled(" - ", Style::default().fg(Color::Gray)),
                        Span::styled(artists, Style::default().fg(Color::Cyan)),
                        Span::styled(" (", Style::default().fg(Color::Gray)),
                        Span::styled(&track.album.name, Style::default().fg(Color::Magenta)),
                        Span::styled(")", Style::default().fg(Color::Gray)),
                    ]);
                    ListItem::new(content)
                })
                .collect();

            let list = List::new(items)
                .block(Block::default().borders(Borders::ALL))
                .highlight_style(Style::default().fg(Color::Black).bg(Color::Green))
                .highlight_symbol("► ");

            f.render_stateful_widget(list, chunks[1], &mut self.favorites_list_state.clone());
        } else {
            let no_favorites = Paragraph::new("No se encontraron canciones favoritas")
                .style(Style::default().fg(Color::Yellow))
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL));

            f.render_widget(no_favorites, chunks[1]);
        }
    }

    fn render_footer(&self, f: &mut Frame, area: Rect) {
        let footer_text = if let Some(ref error) = self.error_message {
            vec![Line::from(vec![
                Span::styled("❌ Error: ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                Span::styled(error, Style::default().fg(Color::Red)),
            ])]
        } else if let Some(ref success) = self.success_message {
            vec![Line::from(vec![
                Span::styled("✅ ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::styled(success, Style::default().fg(Color::Green)),
            ])]
        } else {
            vec![Line::from(vec![
                Span::styled("Estado: ", Style::default().fg(Color::Cyan)),
                Span::styled("Listo", Style::default().fg(Color::Green)),
                Span::styled(" | ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("Actualizado: {:.1}s", self.last_update.elapsed().as_secs_f32()),
                    Style::default().fg(Color::Gray),
                ),
            ])]
        };

        let footer = Paragraph::new(footer_text)
            .alignment(Alignment::Left)
            .block(Block::default().borders(Borders::ALL));

        f.render_widget(footer, area);
    }

    fn render_search_popup(&self, f: &mut Frame) {
        let popup_area = Self::centered_rect(60, 20, f.size());
        f.render_widget(Clear, popup_area);

        let input_text = if self.search_input.is_empty() {
            "Escribe para buscar..."
        } else {
            &self.search_input
        };

        let input = Paragraph::new(input_text)
            .style(Style::default().fg(if self.search_input.is_empty() { Color::Gray } else { Color::White }))
            .block(Block::default().title("Buscar Canciones").borders(Borders::ALL));

        f.render_widget(input, popup_area);
    }

    fn render_volume_popup(&self, f: &mut Frame) {
        let popup_area = Self::centered_rect(40, 15, f.size());
        f.render_widget(Clear, popup_area);

        let input_text = if self.volume_input.is_empty() {
            "0-100"
        } else {
            &self.volume_input
        };

        let input = Paragraph::new(input_text)
            .style(Style::default().fg(if self.volume_input.is_empty() { Color::Gray } else { Color::White }))
            .block(Block::default().title("Volumen (%)").borders(Borders::ALL));

        f.render_widget(input, popup_area);
    }

    fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ])
            .split(r);

        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ])
            .split(popup_layout[1])[1]
    }

    fn format_duration(ms: i64) -> String {
        let seconds = ms / 1000;
        let minutes = seconds / 60;
        let remaining_seconds = seconds % 60;
        format!("{}:{:02}", minutes, remaining_seconds)
    }
} 