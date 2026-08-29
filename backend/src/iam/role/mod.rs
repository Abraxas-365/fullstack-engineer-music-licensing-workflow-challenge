pub mod adapters;
pub mod model;
mod error;
mod port;
mod service;

pub use error::RoleError;
pub use model::{
    AssignRoleRequest, CreateRoleRequest, Role, RoleResponse, UpdateRoleRequest, UserRole,
    UserRolesResponse,
};
pub use port::RoleRepository;
pub use service::RoleService;
