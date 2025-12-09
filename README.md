Software Renderer written in Rust

What's new:

[09.12.2025] Lighting

![Lighting on mesh](https://i.imgur.com/5JTeZGr.png)

[07.12.2025] Mesh rendering and ability to render text with simplest font

![Textured mesh](https://i.imgur.com/FqO1n4j.png)

[05.12.2025] Textured triangle

![Textured triangle](https://i.imgur.com/Vr9KEl5.png)

[04.12.2025] I'm currently in the middle of implementing another software rasterizer, following the fantastic tutorial series by thebennybox on YouTube (https://www.youtube.com/playlist?list=PLEETnX-uPtBUbVOok816vTl1K9vV1GgH5). This approach is more practical compared to Computer Graphics from Scratch by Gabriel Gambetta. What I mean is that it uses algorithms and techniques closely resembling those found in real graphics APIs and GPU pipelines - such as scan conversion, fill conventions, projection matrices, and so on (though everything is still implemented in software).

As a result, the rendered output looks slightly cleaner (no fuzzy or "hairy" textures and edges) and the implementation is more optimized. However, the tutorial sometimes skips over detailed explanations of certain steps. For that reason, I'd personally recommend attempting it only after watching some solid introductory lectures on computer graphics - like the excellent "Computer Graphics, Fall 2009" course from UC Davis.

Progress for now: THE CLASSIC - colored rotating triangle:

![Rotating colored triangle](https://i.imgur.com/VSWbNn2.png)

---

[16.11.2022] Up to this point almost everything was based on the amazing articles "Computer Graphics from Scratch" by Gabriel Gambetta (which recently became a full fledged book but still accessble online here https://gabrielgambetta.com/computer-graphics-from-scratch/index.html). Very recommended for the complete beginners in computer graphics (like myself).


[27.07.2019] Textures

![Textures](https://i.imgur.com/Iu7rrnb.png)

[23.07.2019] Phong Shading model

![Phong Shading](https://i.imgur.com/uL3Ar1Q.png)

[04.05.2019] Gouraud Shading model

![Gouraud Shading](https://i.imgur.com/9GumSIM.png)

[09.04.2019] Point lights and specular component

![Point light](https://i.imgur.com/75Gl27c.png)

[09.04.2019] First attempt at shading (flat, ambient + diffuse only)

![Flat Shading](https://i.imgur.com/veKmcQs.png)
![Flat Shading](https://i.imgur.com/nOlGvw7.png)

[31.03.2019] DepthBuffer implementation

![Depth Bufer](https://i.imgur.com/eZD6l52.png)

[30.12.2018] Split triangles when clipping

![Split triangles](https://i.imgur.com/p5V7rXe.png)

[29.12.2018] Clipping by whole polygon

![Clipping](https://i.imgur.com/HCwInzK.png)

[12.04.2018] Rasterization: model, instances and transforms

![Spheres](https://i.imgur.com/IKh85c9.png)

[18.03.2018] Ray tracing: reflections

![Spheres](https://i.imgur.com/ZjsEhWZ.png)

[13.03.2018] Ray tracing: shadows

![Spheres](https://i.imgur.com/09iosHR.png)

[12.03.2018] Ray tracing: add specular component

![Spheres](https://i.imgur.com/1FCHmB7.png)

[10.03.2018] Ray tracing: "bumpy" spheres and point light

![Spheres](https://i.imgur.com/kwgm2KI.png)

[10.03.2018] Ray tracing with shading (ambient + diffuse)

![Spheres](https://i.imgur.com/iNcXBbA.png)

[08.03.2018] Basic ray tracing (no shading)

![Spheres](https://i.imgur.com/FudprAe.png)

[23.02.2018] Read models from PLY2 format

![Twirl](https://i.imgur.com/edMp9HJ.png)
![Twirl](https://i.imgur.com/Tn7ecpQ.png)

[18.02.2018] Rotated cube (parametrized model)

![Faces](https://i.imgur.com/pUH6ykZ.png)

[18.02.2018] Face visibility detection

![Faces](https://i.imgur.com/OHw3Hxr.png)

[12.02.2018] Wirefame cube

![Cube](https://i.imgur.com/SM7Ofnk.png)
