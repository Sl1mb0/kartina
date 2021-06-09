``` 
 _  _    __    ____  ____  ____  _  _    __   
( )/ )  /__\  (  _ \(_  _)(_  _)( \( )  /__\  
 )  (  /(__)\  )   /  )(   _)(_  )  (  /(__)\ 
(_)\_)(__)(__)(_)\_) (__) (____)(_)\_)(__)(__)

```

Contributors: Timothy Maloney, Andrey Pushkarev

# About
Kartina is an open-source art project that combines my love of deep house music and programming. 
The inspiration for this project comes from the song "Can I Take A Picture With You" by Andrey Pushkarev. 
My goal for this project is to demonstrate that art and computation are an ideal marriage of form and function.

Kartina is a shader that uses decoded mp3 frames to change the color of each individual triangle that composes a sphere rendered by the shader. 
Multi-threading is used to play the song in the background, decode mp3 data, and render the sphere whose colors are determined by the decoded mp3 frame data. 
A single mp3 `Frame` from the mp3 file is decoded, that `Frame` is passed to a `State` method, which uses the decoded mp3 `Frame` to draw 
a sphere whose individual triangle colors are determined by the decoded mp3 frame data.

A decoded mp3 frame in this context refers to an array of `u16`; admittedly, I don't know much about the technical aspects of mp3 file formats and how they 
are encoded, which is why only the numerical data is used.

Since the project consists of a vertex shader and a fragment shader, individual GLSL files must also be compiled when building the project. 
To do this, I've included a build script: `build.rs` that is used to compile the shaders. This makes building and running the project much simpler.
The build script simply compiles each shader and write the compiled shader to a specified path.

# Building
To build the project, first clone the repo. From within the cloned repo type: `cargo build`. Then, to run the project, type: `cargo run`.

## Enjoy.

# Testing
All tests are stored in the `/tests` directory. All rust code in `/src/state` is tested, and I also wrote some tests for the components I used
from the `minimp3` and `play` crates. `wgpu` is a well-documented crate, and there are tons of resources online about it; which is my rationale for why I chose not to test
the `wgpu` components of this project.

# Licensing
This repo contains two licenses. The project itself is released under the _ license. The song, however, is included under the _ creative commons license. I did this to
project Andrey, as I recieved from him to use his song was via email, and very informal. The project may be released, modified, distributed, and credited to whomever. 
The song can be released and distributed alongside the project, but all song credit must go to Andrey Pushkarev.

# References
()[]
()[]
