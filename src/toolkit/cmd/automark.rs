use scoretracker::library::{database::LibraryDatabaseLock, index::LibraryIndex};

pub fn automark_library_files(index: LibraryIndex, _library: LibraryDatabaseLock) {
    for (_entry_path, _entry_uuid) in index.files {
        todo!()
    }
}
