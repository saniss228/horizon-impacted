# Сборка из исходников

Это независимая модификация Horizon v0.2.92 и собственный WebAssembly-мод. Оригинальный код Impact, OptiFine, игровые JAR и ресурсы Minecraft сюда не входят. Для сборки и тестирования отдельно нужны зависимости Horizon, Git, Rust и загруженные пакеты crates.io. Для запуска нужны отдельно подготовленные ресурсы Horizon 26.3.

## Состав

* `horizon-v0.2.92-native.patch` — все изменения к чистому тегу Horizon v0.2.92; в частности, нативный backend, фильтр XRay и интерфейс мода.
* `mod/` — исходники меню и vendored Horizon mod SDK/WIT под соответствующими лицензиями.
* `tests/` — unit-тесты, модовый host-harness и сценарии Windows/Wine E2E.
* `HORIZON-Cargo.lock` — копия lock-файла исходного Horizon, использованного для сборки; `mod/Cargo.lock` фиксирует зависимости мода.
* `licenses/`, `DEPENDENCIES.md`, `HORIZON-LICENSE.txt` и `third_party/symphonia-sources/` — лицензии, уведомления и исходные дистрибутивы покрытых MPL библиотек.

## Пример сборки на Linux для Windows x64

В примере исходники проекта уже распакованы в текущую папку. Кросс-компиляция требует установленного MinGW-w64, Rust 1.99.1 с поддержкой edition 2024 и targets `wasm32-unknown-unknown`, `x86_64-pc-windows-gnu`; могут понадобиться дополнительные системные зависимости Horizon. Сборка требует несколько гигабайт диска и доступ к зависимостям.

В специальном архиве исходников из Releases оригинальный архив Horizon уже вложен. В исходниках, скачанных из GitHub, вместо распаковки можно выполнить `git clone --branch v0.2.92 https://github.com/horizon-rust/horizon.git horizon-0.2.92`.

```sh
export NATIVE_SRC="$PWD"
tar -xzf horizon-v0.2.92.tar.gz
cd horizon-0.2.92
git apply --check "$NATIVE_SRC/horizon-v0.2.92-native.patch"
git apply "$NATIVE_SRC/horizon-v0.2.92-native.patch"
rustup target add wasm32-unknown-unknown x86_64-pc-windows-gnu
cargo install --path crates/cargo-horizon-mod --locked
```

Соберите и проверьте мод:

```sh
cd "$NATIVE_SRC/mod"
cargo test --locked
cargo horizon-mod build
cargo horizon-mod check target/horizon-mod/impact_horizon_native.hmod
```

Соберите клиент. Пониженная оптимизация уменьшает нагрузку на память; для другого toolchain параметры можно скорректировать.

```sh
cd "$NATIVE_SRC/horizon-0.2.92"
export CARGO_INCREMENTAL=0 CARGO_PROFILE_RELEASE_DEBUG=0
export CARGO_PROFILE_RELEASE_OPT_LEVEL=1 CARGO_PROFILE_RELEASE_CODEGEN_UNITS=256
export CARGO_PROFILE_RELEASE_LTO=false HORIZON_VERSION=0.2.92
export RUSTFLAGS='-C target-feature=+crt-static -C link-arg=-Wl,--no-keep-memory'
cargo build --release --locked -p horizon-client --bin horizon-client \
  --features mods --target x86_64-pc-windows-gnu -j1
mkdir -p "$NATIVE_SRC/client"
cp target/x86_64-pc-windows-gnu/release/horizon-client.exe \
  "$NATIVE_SRC/client/horizon-client-native.exe"
x86_64-w64-mingw32-strip --strip-all "$NATIVE_SRC/client/horizon-client-native.exe"
cp "$NATIVE_SRC/mod/target/horizon-mod/impact_horizon_native.hmod" "$NATIVE_SRC/client/"
cp "$NATIVE_SRC/Start-Impact-Native.cmd" "$NATIVE_SRC/client/"
```

Скрипт ожидает EXE с указанным именем и соседний `.hmod`. Native-канал доверяет только модулю, загруженному через `--dev-mod` с canonical-путём соседнего файла `impact_horizon_native.hmod`. Это проверка расположения, **не криптографическая подпись**. Изменённый клиент запускайте отдельно, не перезаписывая официальную установку; укажите путь к ресурсам, как описано в README.

## Проверки

```sh
cd "$NATIVE_SRC"
cargo test --locked --manifest-path mod/Cargo.toml
python3 tests/test_native_logic.py "$NATIVE_SRC/horizon-0.2.92"
rustc --edition 2024 --test tests/run_nav_unit.rs -o /tmp/impact-nav-tests
/tmp/impact-nav-tests
mkdir -p horizon-0.2.92/crates/horizon-mods/examples
cp tests/native_memory_harness.rs \
  horizon-0.2.92/crates/horizon-mods/examples/impact_native_memory.rs
cd horizon-0.2.92
cargo run --release --locked -p horizon-mods --features runtime --example impact_native_memory -j1 -- \
  "$NATIVE_SRC/mod/target/horizon-mod/impact_horizon_native.hmod"
```

**E2E-сценарии разрушительны.** `tests/windows_stage7_e2e.py` меняет блоки и инвентарь, телепортирует персонажа, создаёт и убивает мобов. Запускайте только на тестовой копии клиента в специально созданном одноразовом creative-мире: укажите `IMPACT_QA_ENDPOINT` и `DISPLAY`; фазы `visual`, `bridge`, `bridge_diagonal`, `bridge_reverse`, `bridge_safety`, `crits`. `tests/windows_stage7_players_e2e.py` дополнительно требует отдельного тестового игрока и `IMPACT_QA_GUEST_ENDPOINT`; фазы `open`, `players`, `persist_add` и `persist_check`. Инструкции есть в заголовках файлов.

Для тестов клиент запускается напрямую с локальным Agent Control endpoint, чтобы создать и обследовать тестовые условия. Обычный скрипт устанавливает `IMPACT_NATIVE_DISABLE_CONTROL=1` на время процесса и не открывает endpoint. Любой локальный процесс с доступом к тестовому endpoint может управлять игрой; не включайте его без необходимости.
