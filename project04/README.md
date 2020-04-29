# cs451_project4
 Project 4 for CS 451 (ThreadPool)

## Encode Usage
  Run

  `cargo run 1 message.txt ppm output`

  1 = thread count

  message.txt = input text file with message to encode

  ppm = directory where at least one valid ppm file is located

  output = an existing directory to write encoded ppm files

## Decode Usage
  Run

  `cargo run 1 input`

  1 = thread count

  input = an existing directory to read encoded ppm files


## Notes
  You must remove .DS_Store from any directory that is being read from, done by navigating to the directory and running the following:

  `rm .DS_Store`

  This only currently encodes using the first ppm file in the directory at the moment

  There is some random junk data by not adding a null terminating character to the end of each file. This is an unfixed bug

  The directory that you are writing to must exist before writing to it

  The current directory is project04