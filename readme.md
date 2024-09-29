# Baseball Flight simulation with Bevy and Avian

A **Bevy** plugin for simulating baseball flight powered by **Avian** physics based on the umba baseball flight calculator: <https://github.com/AndRoo88/Baseball-Flight-Calculator>;

Simulates the four forces that affect the trajectory of a baseball in flight: **Gravity**, **Drag**, **Magnus Effect**, and **Seam Shifted Wake (SSW)**.

Send the `DisableAerodynamicsEvent` to disable the simulated forces.

Note that the simulations are performed using imperial units instead and also its own coordinate system and NOT bevy's.

## TODO

- [ ] features for each simulation option
- [ ] simulate in metric units
- [ ] simulate in bevy coordinate system
