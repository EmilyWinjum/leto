# Leto
A work-in-progress ECS-based physics engine with minimal crate dependencies and a focus on accurate multi-body simulation as a testbed and learning sandbox for developing novel engine systems.

both [flecs](https://github.com/SanderMertens/flecs) and [Bevy Engine](https://github.com/bevyengine/bevy)'s ECS implementation are primary inspirations for this project, and I heavily recommend checking them out!

## Crates
### Physics:
    Responsible for engine implementations, providing defined components and systems using Leto's ECS patterns.
### ECS:
    Implements an ECS 'world' to store, read, and mutate data via registered systems.

    ECS Derive: 
    - Convenient derivation macros for defining new Components and Systems