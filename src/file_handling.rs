use serde_json::{from_str, to_string_pretty};
use std::{
    fs::File,
    io::{Error, Read, Seek, Write},
    path::PathBuf,
};

use crate::utils::Note;

pub struct FileHandler {
    file_name: String,
    directory_path: PathBuf,
    file_struct: File,
}

impl FileHandler {
    pub fn new(path: PathBuf, name: String) -> Result<Self, Error> {
        // Create a file struct with permissions for both writing and reading.
        let mut file = File::options()
            .read(true)
            .append(true)
            .create(true)
            .open(&path.join(&name))?;

        // Add the required characters for serialization and deserialization to work if the file was just created.
        if file.metadata()?.len() == 0 {
            file.write_all("[]".as_bytes())?;
            file.sync_all()?;
        }
        // Return the newly created FileHandler struct, indicating a successful operation.
        Ok(FileHandler {
            directory_path: path,
            file_name: name,
            file_struct: file,
        })
    }

    fn get_file_content(&mut self) -> Result<String, Error> {
        // Rewind the file's internal cursor, propagating an error if the operation fails.
        self.file_struct.rewind()?;

        let mut file_content = String::new();
        // Store the result of the read_to_string operation in the previously created variable.
        let result = self.file_struct.read_to_string(&mut file_content);
        match result {
            Ok(_) => Ok(file_content),
            Err(error) => Err(error),
        }
    }

    pub fn deserialize_notes(&mut self) -> Result<Vec<Note>, Box<dyn std::error::Error>> {
        match self.get_file_content() {
            // Try to turn the content of the file into a vector of Note structs and return it.
            Ok(content) => match from_str::<Vec<Note>>(&content) {
                Ok(vector) => Ok(vector),
                Err(error) => Err(error.into()),
            },
            Err(error) => Err(error.into()),
        }
    }

    pub fn serialize_notes(&mut self, notes: &Vec<Note>) -> Result<(), Box<dyn std::error::Error>> {
        // Turn the vector of Note structs in memory into a String of JSON data.
        let serialized_notes = to_string_pretty(notes)?;
        // Attempt to store that data in persistent storage.
        match self.update_file_content(serialized_notes) {
            Ok(()) => Ok(()),
            Err(error) => Err(error.into()),
        }
    }

    fn update_file_content(&mut self, content: String) -> Result<(), Error> {
        // Create a new file where the updated data will be stored.
        let new_file_name = self.file_name.clone() + ".tmp";
        let mut new_file = File::create(self.directory_path.join(&new_file_name))?;
        new_file.write_all(content.as_bytes())?;
        new_file.sync_all()?;
        drop(new_file);

        // Rename it, in such a way that if no error has ocurred so far the file at the original
        // location will have the updated data.
        std::fs::rename(
            self.directory_path.join(new_file_name),
            self.directory_path.join(&self.file_name),
        )?;

        // Ensure to send the updated directory metadata from the OS kernel to disk, except on Windows.
        #[cfg(not(windows))]
        File::open(&self.directory_path)?.sync_all()?;

        // Open the new file and give ownership of such struct to the file handler.
        self.file_struct = File::options()
            .read(true)
            .append(true)
            .open(&self.directory_path.join(&self.file_name))?;

        Ok(())
    }

    pub fn add_note(&mut self, note: Note) -> Result<(), Box<dyn std::error::Error>> {
        let mut notes = self.deserialize_notes()?;
        notes.push(note);
        self.serialize_notes(&notes)
    }

    pub fn remove_note(&mut self, id: u32) -> Result<(), Box<dyn std::error::Error>> {
        let mut notes = self.deserialize_notes()?;

        if let Some(target_index) = notes.iter().position(|x| x.id == id) {
            notes.remove(target_index);
            self.serialize_notes(&notes)?;
        }
        Ok(())
    }
}
