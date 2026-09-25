# Full scan (no knowledge about previous state)
1. Find all relevant (.mp4, .png, ...) files in a given directory.
2. Calculate their SHA256 hashes (or fetch from local `library_cache.json` file).
3. Send all calculated/cached SHA256 hashes, along with internal filepaths, for files that are currently present in the directory to the server/database. `/api/library/sync_full`
4. The server should:
    * remove ALL previous references to this library domain in the library URLs from the database,
    * add URLs for the specified files (meaning, all old files that used be in this library and are not present anymore will have their URLs removed, and any files currently present will have their URLs updated),
    * insert new proof records in the case that a SHA256 hash was not found in the database,
    * return proof UUIDs for every file/hash pair in the request.
5. Create a new library index with the returned data.

# Add specific files
1. Calculate their SHA256 hashes (or fetch from local `library_cache.json` file). 
2. Send all calculated/cached SHA256 hashes, along with internal filepaths, for files that are currently present in the directory to the server/database. `/api/library/sync_add`
3. The server should:
    * NOT remove previous references to this library domain in the library URLs from the database,
    * add URLs for the specified files,
    * insert new proof records in the case that a SHA256 hash was not found in the database,
    * return proof UUIDs for every file/hash pair in the request.