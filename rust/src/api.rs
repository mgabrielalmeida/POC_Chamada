//! Funções públicas que o gerador expõe ao Flutter.

/// Confirma que a interface chamou a biblioteca nativa, sem abrir dispositivos.
pub fn verificar_ponte() -> String {
    "Ponte Flutter–Rust conectada.".to_owned()
}
