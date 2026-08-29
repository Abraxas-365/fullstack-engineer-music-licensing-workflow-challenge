use std::sync::Arc;

use chrono::Utc;

use crate::error::AppError;
use crate::kernel::{TrackId, UserId};
use crate::scene::SceneRepository;
use crate::song::SongRepository;

use super::error::TrackError;
use super::model::{CreateTrackRequest, Track, UpdateTrackRequest};
use super::port::TrackRepository;

pub struct TrackService {
    track_repo: Arc<dyn TrackRepository>,
    scene_repo: Arc<dyn SceneRepository>,
    song_repo: Arc<dyn SongRepository>,
}

impl TrackService {
    pub fn new(
        track_repo: Arc<dyn TrackRepository>,
        scene_repo: Arc<dyn SceneRepository>,
        song_repo: Arc<dyn SongRepository>,
    ) -> Self {
        Self {
            track_repo,
            scene_repo,
            song_repo,
        }
    }

    pub async fn create_track(
        &self,
        req: CreateTrackRequest,
        created_by: UserId,
    ) -> Result<Track, AppError> {
        let usage_type = req.validate()?;

        self.scene_repo
            .get_by_id(&req.scene_id)
            .await?
            .ok_or_else(|| AppError::not_found("Scene not found"))?;

        self.song_repo
            .get_by_id(&req.song_id)
            .await?
            .ok_or_else(|| AppError::not_found("Song not found"))?;

        // Check if song is already placed in this scene
        if self
            .track_repo
            .get_by_scene_and_song(&req.scene_id, &req.song_id)
            .await?
            .is_some()
        {
            return Err(TrackError::already_exists());
        }

        let mut track = Track::new(req.scene_id, req.song_id, usage_type, created_by);
        track.notes = req.notes;

        self.track_repo.save(&track).await?;
        Ok(track)
    }

    pub async fn get_track(&self, id: &TrackId) -> Result<Track, AppError> {
        self.track_repo
            .get_by_id(id)
            .await?
            .ok_or_else(|| TrackError::not_found())
    }

    pub async fn list_by_scene(
        &self,
        scene_id: &crate::kernel::SceneId,
    ) -> Result<Vec<Track>, AppError> {
        self.track_repo.list_by_scene(scene_id).await
    }

    pub async fn list_by_song(
        &self,
        song_id: &crate::kernel::SongId,
    ) -> Result<Vec<Track>, AppError> {
        self.track_repo.list_by_song(song_id).await
    }

    pub async fn update_track(
        &self,
        id: &TrackId,
        req: UpdateTrackRequest,
    ) -> Result<Track, AppError> {
        let new_usage = req.validate()?;

        let mut track = self.get_track(id).await?;

        if let Some(usage_type) = new_usage {
            track.usage_type = usage_type;
        }
        if let Some(notes) = req.notes {
            track.notes = Some(notes);
        }
        track.updated_at = Utc::now();

        self.track_repo.update(&track).await?;
        Ok(track)
    }

    pub async fn delete_track(&self, id: &TrackId) -> Result<(), AppError> {
        self.get_track(id).await?;
        self.track_repo.delete(id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel::{LabelId, MovieId, SceneId, SongId, UserId};
    use crate::kernel::{Paginated, PaginationOptions};
    use crate::scene::{Scene, SceneRepository};
    use crate::song::{Song, SongFilter, SongRepository};
    use tokio::sync::Mutex;

    // ========================================================================
    // Mocks
    // ========================================================================

    struct MockTrackRepo {
        tracks: Mutex<Vec<Track>>,
    }
    impl MockTrackRepo {
        fn new() -> Self {
            Self {
                tracks: Mutex::new(Vec::new()),
            }
        }
    }
    #[async_trait::async_trait]
    impl TrackRepository for MockTrackRepo {
        async fn save(&self, track: &Track) -> Result<(), AppError> {
            self.tracks.lock().await.push(track.clone());
            Ok(())
        }
        async fn get_by_id(&self, id: &TrackId) -> Result<Option<Track>, AppError> {
            let tracks = self.tracks.lock().await;
            Ok(tracks.iter().find(|t| t.id == *id).cloned())
        }
        async fn list_by_scene(&self, scene_id: &SceneId) -> Result<Vec<Track>, AppError> {
            let tracks = self.tracks.lock().await;
            Ok(tracks
                .iter()
                .filter(|t| t.scene_id == *scene_id)
                .cloned()
                .collect())
        }
        async fn list_by_song(&self, song_id: &SongId) -> Result<Vec<Track>, AppError> {
            let tracks = self.tracks.lock().await;
            Ok(tracks
                .iter()
                .filter(|t| t.song_id == *song_id)
                .cloned()
                .collect())
        }
        async fn get_by_scene_and_song(
            &self,
            scene_id: &SceneId,
            song_id: &SongId,
        ) -> Result<Option<Track>, AppError> {
            let tracks = self.tracks.lock().await;
            Ok(tracks
                .iter()
                .find(|t| t.scene_id == *scene_id && t.song_id == *song_id)
                .cloned())
        }
        async fn update(&self, track: &Track) -> Result<(), AppError> {
            let mut tracks = self.tracks.lock().await;
            if let Some(t) = tracks.iter_mut().find(|t| t.id == track.id) {
                *t = track.clone();
            }
            Ok(())
        }
        async fn delete(&self, id: &TrackId) -> Result<(), AppError> {
            self.tracks.lock().await.retain(|t| t.id != *id);
            Ok(())
        }
    }

    struct MockSceneRepo {
        scenes: Mutex<Vec<Scene>>,
    }
    impl MockSceneRepo {
        fn new() -> Self {
            Self {
                scenes: Mutex::new(Vec::new()),
            }
        }
        fn with_scene(scene: Scene) -> Self {
            Self {
                scenes: Mutex::new(vec![scene]),
            }
        }
    }
    #[async_trait::async_trait]
    impl SceneRepository for MockSceneRepo {
        async fn save(&self, _: &Scene) -> Result<(), AppError> {
            Ok(())
        }
        async fn get_by_id(&self, id: &SceneId) -> Result<Option<Scene>, AppError> {
            let scenes = self.scenes.lock().await;
            Ok(scenes.iter().find(|s| s.id == *id).cloned())
        }
        async fn list_by_movie(&self, _: &MovieId) -> Result<Vec<Scene>, AppError> {
            Ok(vec![])
        }
        async fn update(&self, _: &Scene) -> Result<(), AppError> {
            Ok(())
        }
        async fn delete(&self, _: &SceneId) -> Result<(), AppError> {
            Ok(())
        }
    }

    struct MockSongRepo {
        songs: Mutex<Vec<Song>>,
    }
    impl MockSongRepo {
        fn new() -> Self {
            Self {
                songs: Mutex::new(Vec::new()),
            }
        }
        fn with_song(song: Song) -> Self {
            Self {
                songs: Mutex::new(vec![song]),
            }
        }
    }
    #[async_trait::async_trait]
    impl SongRepository for MockSongRepo {
        async fn save(&self, _: &Song) -> Result<(), AppError> {
            Ok(())
        }
        async fn get_by_id(&self, id: &SongId) -> Result<Option<Song>, AppError> {
            let songs = self.songs.lock().await;
            Ok(songs.iter().find(|s| s.id == *id).cloned())
        }
        async fn find(
            &self,
            _: &PaginationOptions,
            _: &SongFilter,
        ) -> Result<Paginated<Song>, AppError> {
            Ok(Paginated::new(vec![], 1, 10, 0))
        }
        async fn update(&self, _: &Song) -> Result<(), AppError> {
            Ok(())
        }
        async fn delete(&self, _: &SongId) -> Result<(), AppError> {
            Ok(())
        }
        async fn list_by_artist(&self, _: &UserId) -> Result<Vec<Song>, AppError> {
            Ok(vec![])
        }
        async fn list_by_label(&self, _: &LabelId) -> Result<Vec<Song>, AppError> {
            Ok(vec![])
        }
    }

    fn make_scene() -> Scene {
        Scene::new(MovieId::new(), "Opening".into(), 1, 0, 120)
    }

    fn make_song() -> Song {
        Song::new("Test Song".into(), UserId::new(), None, 240)
    }

    fn make_svc(
        track_repo: MockTrackRepo,
        scene_repo: MockSceneRepo,
        song_repo: MockSongRepo,
    ) -> TrackService {
        TrackService::new(
            Arc::new(track_repo),
            Arc::new(scene_repo),
            Arc::new(song_repo),
        )
    }

    fn create_req(scene_id: SceneId, song_id: SongId) -> CreateTrackRequest {
        CreateTrackRequest {
            scene_id,
            song_id,
            usage_type: "BACKGROUND".into(),
            notes: None,
        }
    }

    // ========================================================================
    // create_track
    // ========================================================================

    #[tokio::test]
    async fn create_track_success() {
        let scene = make_scene();
        let song = make_song();
        let svc = make_svc(
            MockTrackRepo::new(),
            MockSceneRepo::with_scene(scene.clone()),
            MockSongRepo::with_song(song.clone()),
        );
        let track = svc
            .create_track(create_req(scene.id.clone(), song.id.clone()), UserId::new())
            .await
            .unwrap();
        assert_eq!(track.scene_id, scene.id);
        assert_eq!(track.song_id, song.id);
        assert_eq!(track.usage_type, super::super::model::UsageType::Background);
        assert!(track.notes.is_none());
    }

    #[tokio::test]
    async fn create_track_with_notes() {
        let scene = make_scene();
        let song = make_song();
        let svc = make_svc(
            MockTrackRepo::new(),
            MockSceneRepo::with_scene(scene.clone()),
            MockSongRepo::with_song(song.clone()),
        );
        let mut req = create_req(scene.id.clone(), song.id.clone());
        req.notes = Some("Plays softly in background".into());
        let track = svc.create_track(req, UserId::new()).await.unwrap();
        assert_eq!(track.notes.as_deref(), Some("Plays softly in background"));
    }

    #[tokio::test]
    async fn create_track_featured_usage() {
        let scene = make_scene();
        let song = make_song();
        let svc = make_svc(
            MockTrackRepo::new(),
            MockSceneRepo::with_scene(scene.clone()),
            MockSongRepo::with_song(song.clone()),
        );
        let mut req = create_req(scene.id.clone(), song.id.clone());
        req.usage_type = "FEATURED".into();
        let track = svc.create_track(req, UserId::new()).await.unwrap();
        assert_eq!(track.usage_type, super::super::model::UsageType::Featured);
    }

    #[tokio::test]
    async fn create_track_invalid_usage_type() {
        let scene = make_scene();
        let song = make_song();
        let svc = make_svc(
            MockTrackRepo::new(),
            MockSceneRepo::with_scene(scene.clone()),
            MockSongRepo::with_song(song.clone()),
        );
        let mut req = create_req(scene.id.clone(), song.id.clone());
        req.usage_type = "INVALID".into();
        let err = svc.create_track(req, UserId::new()).await.unwrap_err();
        assert_eq!(err.code, "VALIDATION_ERROR");
    }

    #[tokio::test]
    async fn create_track_scene_not_found() {
        let song = make_song();
        let svc = make_svc(
            MockTrackRepo::new(),
            MockSceneRepo::new(),
            MockSongRepo::with_song(song.clone()),
        );
        let err = svc
            .create_track(create_req(SceneId::new(), song.id.clone()), UserId::new())
            .await
            .unwrap_err();
        assert_eq!(err.code, "NOT_FOUND");
    }

    #[tokio::test]
    async fn create_track_song_not_found() {
        let scene = make_scene();
        let svc = make_svc(
            MockTrackRepo::new(),
            MockSceneRepo::with_scene(scene.clone()),
            MockSongRepo::new(),
        );
        let err = svc
            .create_track(create_req(scene.id.clone(), SongId::new()), UserId::new())
            .await
            .unwrap_err();
        assert_eq!(err.code, "NOT_FOUND");
    }

    #[tokio::test]
    async fn create_track_duplicate_song_in_scene() {
        let scene = make_scene();
        let song = make_song();
        let svc = make_svc(
            MockTrackRepo::new(),
            MockSceneRepo::with_scene(scene.clone()),
            MockSongRepo::with_song(song.clone()),
        );
        svc.create_track(create_req(scene.id.clone(), song.id.clone()), UserId::new())
            .await
            .unwrap();
        let err = svc
            .create_track(create_req(scene.id.clone(), song.id.clone()), UserId::new())
            .await
            .unwrap_err();
        assert_eq!(err.code, "track.already_exists");
    }

    // ========================================================================
    // get_track
    // ========================================================================

    #[tokio::test]
    async fn get_track_success() {
        let scene = make_scene();
        let song = make_song();
        let svc = make_svc(
            MockTrackRepo::new(),
            MockSceneRepo::with_scene(scene.clone()),
            MockSongRepo::with_song(song.clone()),
        );
        let created = svc
            .create_track(create_req(scene.id.clone(), song.id.clone()), UserId::new())
            .await
            .unwrap();
        let found = svc.get_track(&created.id).await.unwrap();
        assert_eq!(found.id, created.id);
    }

    #[tokio::test]
    async fn get_track_not_found() {
        let svc = make_svc(
            MockTrackRepo::new(),
            MockSceneRepo::new(),
            MockSongRepo::new(),
        );
        let err = svc.get_track(&TrackId::new()).await.unwrap_err();
        assert_eq!(err.code, "track.not_found");
    }

    // ========================================================================
    // list_by_scene / list_by_song
    // ========================================================================

    #[tokio::test]
    async fn list_by_scene_success() {
        let scene = make_scene();
        let song1 = make_song();
        let song2 = make_song();
        let song_repo = MockSongRepo {
            songs: Mutex::new(vec![song1.clone(), song2.clone()]),
        };
        let svc = make_svc(
            MockTrackRepo::new(),
            MockSceneRepo::with_scene(scene.clone()),
            song_repo,
        );
        svc.create_track(
            create_req(scene.id.clone(), song1.id.clone()),
            UserId::new(),
        )
        .await
        .unwrap();
        let mut req2 = create_req(scene.id.clone(), song2.id.clone());
        req2.usage_type = "CREDITS".into();
        svc.create_track(req2, UserId::new()).await.unwrap();

        let tracks = svc.list_by_scene(&scene.id).await.unwrap();
        assert_eq!(tracks.len(), 2);
    }

    #[tokio::test]
    async fn list_by_song_success() {
        let scene1 = make_scene();
        let scene2 = make_scene();
        let song = make_song();
        let scene_repo = MockSceneRepo {
            scenes: Mutex::new(vec![scene1.clone(), scene2.clone()]),
        };
        let svc = make_svc(
            MockTrackRepo::new(),
            scene_repo,
            MockSongRepo::with_song(song.clone()),
        );
        svc.create_track(
            create_req(scene1.id.clone(), song.id.clone()),
            UserId::new(),
        )
        .await
        .unwrap();
        svc.create_track(
            create_req(scene2.id.clone(), song.id.clone()),
            UserId::new(),
        )
        .await
        .unwrap();

        let tracks = svc.list_by_song(&song.id).await.unwrap();
        assert_eq!(tracks.len(), 2);
    }

    #[tokio::test]
    async fn list_by_scene_empty() {
        let svc = make_svc(
            MockTrackRepo::new(),
            MockSceneRepo::new(),
            MockSongRepo::new(),
        );
        let tracks = svc.list_by_scene(&SceneId::new()).await.unwrap();
        assert!(tracks.is_empty());
    }

    // ========================================================================
    // update_track
    // ========================================================================

    #[tokio::test]
    async fn update_track_usage_type() {
        let scene = make_scene();
        let song = make_song();
        let svc = make_svc(
            MockTrackRepo::new(),
            MockSceneRepo::with_scene(scene.clone()),
            MockSongRepo::with_song(song.clone()),
        );
        let created = svc
            .create_track(create_req(scene.id.clone(), song.id.clone()), UserId::new())
            .await
            .unwrap();
        let updated = svc
            .update_track(
                &created.id,
                UpdateTrackRequest {
                    usage_type: Some("FEATURED".into()),
                    notes: None,
                },
            )
            .await
            .unwrap();
        assert_eq!(updated.usage_type, super::super::model::UsageType::Featured);
    }

    #[tokio::test]
    async fn update_track_notes() {
        let scene = make_scene();
        let song = make_song();
        let svc = make_svc(
            MockTrackRepo::new(),
            MockSceneRepo::with_scene(scene.clone()),
            MockSongRepo::with_song(song.clone()),
        );
        let created = svc
            .create_track(create_req(scene.id.clone(), song.id.clone()), UserId::new())
            .await
            .unwrap();
        let updated = svc
            .update_track(
                &created.id,
                UpdateTrackRequest {
                    usage_type: None,
                    notes: Some("Loud during chase".into()),
                },
            )
            .await
            .unwrap();
        assert_eq!(updated.notes.as_deref(), Some("Loud during chase"));
        assert_eq!(
            updated.usage_type,
            super::super::model::UsageType::Background
        );
    }

    #[tokio::test]
    async fn update_track_not_found() {
        let svc = make_svc(
            MockTrackRepo::new(),
            MockSceneRepo::new(),
            MockSongRepo::new(),
        );
        let err = svc
            .update_track(
                &TrackId::new(),
                UpdateTrackRequest {
                    usage_type: Some("FEATURED".into()),
                    notes: None,
                },
            )
            .await
            .unwrap_err();
        assert_eq!(err.code, "track.not_found");
    }

    #[tokio::test]
    async fn update_track_invalid_usage_type() {
        let scene = make_scene();
        let song = make_song();
        let svc = make_svc(
            MockTrackRepo::new(),
            MockSceneRepo::with_scene(scene.clone()),
            MockSongRepo::with_song(song.clone()),
        );
        let created = svc
            .create_track(create_req(scene.id.clone(), song.id.clone()), UserId::new())
            .await
            .unwrap();
        let err = svc
            .update_track(
                &created.id,
                UpdateTrackRequest {
                    usage_type: Some("NOPE".into()),
                    notes: None,
                },
            )
            .await
            .unwrap_err();
        assert_eq!(err.code, "VALIDATION_ERROR");
    }

    // ========================================================================
    // delete_track
    // ========================================================================

    #[tokio::test]
    async fn delete_track_success() {
        let scene = make_scene();
        let song = make_song();
        let svc = make_svc(
            MockTrackRepo::new(),
            MockSceneRepo::with_scene(scene.clone()),
            MockSongRepo::with_song(song.clone()),
        );
        let created = svc
            .create_track(create_req(scene.id.clone(), song.id.clone()), UserId::new())
            .await
            .unwrap();
        svc.delete_track(&created.id).await.unwrap();
        let err = svc.get_track(&created.id).await.unwrap_err();
        assert_eq!(err.code, "track.not_found");
    }

    #[tokio::test]
    async fn delete_track_not_found() {
        let svc = make_svc(
            MockTrackRepo::new(),
            MockSceneRepo::new(),
            MockSongRepo::new(),
        );
        let err = svc.delete_track(&TrackId::new()).await.unwrap_err();
        assert_eq!(err.code, "track.not_found");
    }
}
