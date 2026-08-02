use scoretracker::data::library::{database::LibraryDatabase, index::LibraryIndex};

pub fn automark_library_files(index: LibraryIndex, _library: LibraryDatabase) {
    for (_entry_path, _entry_uuid) in index.files {
        todo!()
    }
}
