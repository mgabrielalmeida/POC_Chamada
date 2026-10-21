# Mantém a compilação Rust explícita e reaproveita o cache do workspace.
get_filename_component(CHAMADA_RAIZ "${CMAKE_CURRENT_SOURCE_DIR}/../.." ABSOLUTE)
set(CHAMADA_PERFIL "$<IF:$<CONFIG:Debug>,dev,release>")
set(CHAMADA_SAIDA "$<IF:$<CONFIG:Debug>,debug,release>")
add_custom_target(chamada_rust ALL
  COMMAND cargo build --manifest-path "${CHAMADA_RAIZ}/Cargo.toml"
    --target-dir "${CHAMADA_RAIZ}/target" -p chamada --profile "${CHAMADA_PERFIL}"
  WORKING_DIRECTORY "${CHAMADA_RAIZ}"
  VERBATIM)
add_dependencies(${BINARY_NAME} chamada_rust)
install(FILES "${CHAMADA_RAIZ}/target/${CHAMADA_SAIDA}/chamada.dll"
  DESTINATION "${INSTALL_BUNDLE_LIB_DIR}" COMPONENT Runtime)
