use std::sync::Arc;

use chrono::Utc;

use crate::error::AppError;
use crate::iam::user::UserRepository;
use crate::kernel::{LabelId, Paginated, PaginationOptions, SongId, UserId};
use crate::label::{LabelRepository, LabelRole};

use super::error::SongError;
use super::model::{CreateSongRequest, Song, SongFilter, UpdateSongRequest};
use super::port::SongRepository;

pub struct SongService {
    song_repo: Arc<dyn SongRepository>,
    user_repo: Arc<dyn UserRepository>,
    label_repo: Arc<dyn LabelRepository>,
}

impl SongService {
    pub fn new(
        song_repo: Arc<dyn SongRepository>,
        user_repo: Arc<dyn UserRepository>,
        label_repo: Arc<dyn LabelRepository>,
    ) -> Self {
        Self {
            song_repo,
            user_repo,
            label_repo,
        }
    }

    pub async fn create_song(&self, req: CreateSongRequest) -> Result<Song, AppError> {
        req.validate()?;

        // Verify artist exists
        self.user_repo
            .get_by_id(&req.artist_id)
            .await?
            .ok_or_else(|| AppError::not_found("Artist not found"))?;

        // Verify label exists and artist belongs to it as ARTIST
        if let Some(ref label_id) = req.label_id {
            self.label_repo
                .get_by_id(label_id)
                .await?
                .ok_or_else(|| AppError::not_found("Label not found"))?;

            let member = self
                .label_repo
                .get_member(label_id, &req.artist_id)
                .await?
                .ok_or_else(|| {
                    AppError::validation("Artist is not a member of the specified label")
                })?;

            if member.role != LabelRole::Artist {
                return Err(AppError::validation(
                    "User is not registered as an artist in the specified label",
                ));
            }
        }

        let mut song = Song::new(
            req.title,
            req.artist_id,
            req.label_id,
            req.duration_seconds,
        );
        song.album = req.album;
        song.genre = req.genre;
        song.isrc = req.isrc;

        self.song_repo.save(&song).await?;
        Ok(song)
    }

    pub async fn get_song(&self, id: &SongId) -> Result<Song, AppError> {
        self.song_repo
            .get_by_id(id)
            .await?
            .ok_or_else(|| SongError::not_found())
    }

    pub async fn find_songs(
        &self,
        opts: &PaginationOptions,
        filter: &SongFilter,
    ) -> Result<Paginated<Song>, AppError> {
        self.song_repo.find(opts, filter).await
    }

    pub async fn update_song(
        &self,
        id: &SongId,
        req: UpdateSongRequest,
    ) -> Result<Song, AppError> {
        req.validate()?;

        let mut song = self.get_song(id).await?;

        if let Some(title) = req.title {
            song.title = title;
        }
        if let Some(album) = req.album {
            song.album = Some(album);
        }
        if let Some(genre) = req.genre {
            song.genre = Some(genre);
        }
        if let Some(isrc) = req.isrc {
            song.isrc = Some(isrc);
        }
        if let Some(duration) = req.duration_seconds {
            song.duration_seconds = duration;
        }
        song.updated_at = Utc::now();

        self.song_repo.update(&song).await?;
        Ok(song)
    }

    pub async fn delete_song(&self, id: &SongId) -> Result<(), AppError> {
        self.get_song(id).await?;
        self.song_repo.delete(id).await
    }

    pub async fn list_by_artist(&self, artist_id: &UserId) -> Result<Vec<Song>, AppError> {
        self.song_repo.list_by_artist(artist_id).await
    }

    pub async fn list_by_label(&self, label_id: &LabelId) -> Result<Vec<Song>, AppError> {
        self.song_repo.list_by_label(label_id).await
    }
}
