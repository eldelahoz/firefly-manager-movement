# Constitución — firefly-manager-movement

Seis principios innegociables. Todo spec, plan y cambio de código debe cumplirlos; un cambio que los contradiga se rechaza o se reescribe.

1. **Stack simple**: ninguna dependencia nueva (crate Rust, paquete npm) sin justificación explícita en el spec del cambio que la introduce.
   *Verificable:* toda dependencia nueva aparece justificada en su spec antes de entrar en el código.

2. **Spec antes que código**: todo cambio de comportamiento empieza en `specs/<nombre>/spec.md`; el commit/PR correspondiente remite explícitamente a ese spec.
   *Verificable:* ningún commit que cambie comportamiento sin referencia a su spec.

3. **Lógica ≠ interfaz**: la lógica de negocio vive en `domain`/`application` (Rust), jamás en componentes React; las dependencias apuntan siempre hacia dentro.
   *Verificable:* sin lógica de negocio en `frontend/src`; `domain` sin dependencias externas.

4. **Tests como gate**: nada se declara terminado sin `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` y `pnpm lint && pnpm test && pnpm build` en verde.
   *Verificable:* los gates se ejecutan y pasan antes de marcar cualquier tarea como hecha.

5. **Persistencia segura**: importes siempre enteros en cents (BIGINT); toda migración con `up.sql` + `down.sql`; cero mutaciones automáticas sobre Firefly III.
   *Verificable:* sin floats para dinero; toda mutación FF3 nace de una acción explícita del usuario.

6. **Idioma**: código, identificadores, comentarios y commits en inglés; UI con i18n es/en/fr; specs y plans en español.
   *Verificable:* diffs sin español en código/commits; claves i18n presentes en los tres idiomas.
