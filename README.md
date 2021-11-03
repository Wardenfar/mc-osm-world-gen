# Minecraft Real world generator (OpenStreetMap)

- 3000 chunk/sec
- 150k chunks in 1 minute

## Run

```shell
git clone ...
cd ...
cargo run --release -- <file.pbf>
```

1) Output in the folder : ./world
2) Before each generation : clear ./world/region/*