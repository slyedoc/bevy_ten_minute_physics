# Bevy Ten Minutes Physics

These are examples of how to use the [Bevy](https://bevyengine.org/) engine to create a simple physics simulations based on Matthias Muller's amazing [Ten Minute Physics](https://matthias-research.github.io/pages/tenMinutePhysics/index.html) tutorials, go check them out.

## Why?

I have playing around with creating a bevy native physics engine for a while, and while I have learned alot, they haven't been usable.  My lastest issue been has implementing a global solver for contraints.  Even with simple contrains it feels like you need a PHD (which I don't have) and so many advance optimazations to be performant, (sparse matrix representations for example) that the KISS principle is no where in sight.  That's why I am hopeful for XPBD, it seems to be a much simpler approach to solving the same problem.

## Tutorials

> These are not polished bevy or rust examples, I have done just enough to recreate Muller's examples.  I do plan on writing a xpbd physics plugin for bevy, this is not that plugin.

### 1. 2d physics
<img src="/docs/images/20230335-023526.png"  height="400" />

### 2. 3d physics
<img src="docs/images/20230338-023802.png" height="400" />

### 3. 2d ball collision
<img src="docs/images/20230302-230216.png" height="400" />

### 4. Pinball
<img src="docs/images/20230351-015132.png" height="400" />

### 5. Beads
<img src="docs/images/20230351-135141.png" height="400" />

### 6. Pendulum
<img src="docs/images/20230357-025758.png" height="400" />

### 7. 3d vector math
No code for this one.

### 8. User Interaction
<img src="docs/images/20230324-202430.png" height="400" />

### 9. XPBD
No code for this one.

### 10. Softbodies
<img src="docs/images/20230337-003714.png" height="400" />

### 11. Spatial Hashing
<img src="docs/images/20230325-232521.png" height="400" />

### 12. Speedup Softbodies

## Credits

- [Matthias Muller](https://matthias-research.github.io/pages/tenMinutePhysics/index.html) for the amazing tutorials, code, and papers.