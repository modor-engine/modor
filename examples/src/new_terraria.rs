const SCOPE: Scope = Scope::new("root");

fn main() {
    modor_graphics::run(Level::Info, root);
}

fn root(app: &mut App) {
    app.run(chunks);
}

fn chunks(app: &mut App, chunks: &mut Chunks) {
    if chunks.is_initialized {
        for chunk in chunks(app, character_chunk) {
            let scope = *chunks.0.entry(chunk.coords).or_insert_with(Scope::unique);
            new_chunk(app, scope, chunk);
        }
    } else {
        for (coords, scope) in &chunks.0 {
            existing_chunk(app, scope, coords)
        }
    }
}

fn new_chunk(app: &mut App, scope: Scope, chunk: ChunkData) {}

fn existing_chunk(app: &mut App, scope: Scope, chunk: ChunkCoords) {}

#[derive(Default)]
struct Chunks {
    scopes: FxHashMap<ChunkCoords, Scope>,
    is_initialized: bool,
}
