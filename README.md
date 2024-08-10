This crate provides easy to use way to make systems that operate on a single entity.

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

