This crate provides easy to use way to make systems that operate on a single entity.

# Warning
Currently there is a bug, individual entity systems don't store their state.
State is being shared across all entities that are being iterated by the system that resulted from `into_system` call.
That means that `Local`, `EntityReader` and other `SystemParam`'s, that
rely on it's state property to be preserved between system runs, won't work correctly.
It is a huge footgun, so the work on fixing it is being done.

After fix, the API will change, user would need to explicitly specify the entities this system is allowed to run on.
That means, you most probably would need to be able to mutate systems at 
runtime - something that bevy currently doesn't support. 
I also work on a crate that will be able to provide you with such functionality - it is not published yet.

# Example
(of current functionality)

```rust
#[derive(Component)]
struct MyMarkerComponent;

fn my_entity_system(data: Data<&mut Transform, With<MyMarkerComponent>>, mut commands: Commands) {
    *data.item += 10;
    commands.spawn(Transform::from_translation(data.item.translation));
}

fn my_entity_system_with_input(input: In<Vec2>, data: Data<&mut Transform>) {
    *data.item += input;
}

app.add_systems(Update, (
    my_entity_system.into_system(), 
    my_entity_system_with_input.into_system()
));
```

