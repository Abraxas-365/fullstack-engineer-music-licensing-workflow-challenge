pub mod adapters;
mod error;
mod model;
mod port;
mod service;

pub use error::MovieError;
pub use model::{CreateMovieRequest, Movie, MovieFilter, MovieResponse, UpdateMovieRequest};
pub use port::MovieRepository;
pub use service::MovieService;
