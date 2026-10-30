use thiserror::Error;

use crate::{MapObject, Position};

#[derive(PartialEq, Error, Debug)]
pub enum MapGenError {
    #[error("UnknownError")]
    UnknownError,
    #[error("Out of bounds - {position:?}")]
    OutOfBounds { position: Position },
    #[error(
        "Collision - {position:?}. Try place object {new_obj:?} | Existing object {existing_obj:?}"
    )]
    Occupied {
        position: Position,
        new_obj: MapObject,
        existing_obj: MapObject,
    },
    #[error("Can't find a free position on the map")]
    FreePositionNotFound,
}
