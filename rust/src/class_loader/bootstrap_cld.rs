use std::{ptr::NonNull, sync::OnceLock};

use crate::{
    class_loader::{class_path::ClassPath, cld::ClassLoaderData, load_error::LoadResult},
    oops::klass::Klass,
};

