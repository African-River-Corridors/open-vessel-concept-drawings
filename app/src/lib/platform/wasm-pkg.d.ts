// Stand-in types for the wasm-bindgen output (app/wasm-pkg/, built by tools/build_web.sh, not in git),
// so `npm run check` works before the WebAssembly build has run. The real .d.ts wins when it exists.
declare module "*/wasm-pkg/ovd_web.js" {
  export class Session {
    constructor();
    snapshot(): string;
    input_schema(): string;
    evaluate(): string;
    new_design(): string;
    open_text(name: string, text: string): string;
    save_text(): string;
    saved(name: string): string;
    is_dirty(): boolean;
    set_input(option: string, name: string, value_json: string): string;
    import_offsets(option: string, csv: string): string;
    reset_input(option: string, name: string): string;
    set_active_option(option: string): string;
    set_formation(option: string, kind: string, abreast?: number, in_line?: number): string;
    add_option(): string;
    duplicate_option(option: string): string;
    rename_option(option: string, new_name: string): string;
    delete_option(option: string): string;
    load_profile_text(text: string): string;
    use_built_in_profile(): string;
    drawing_svg(option: string, view: string): string;
    drawing_pdf(option: string, view: string): Uint8Array;
    drawing_dxf(option: string, view: string): Uint8Array;
    export_gltf(option: string): Uint8Array;
    mesh_hash(option: string): string;
    mesh_positions(option: string): Float32Array;
    mesh_indices(option: string): Uint32Array;
    mesh_groups(option: string): string;
    section_at(option: string, x: number): string;
    space_info(option: string, name: string): string;
    deck_items(option: string): string;
    propulsors(option: string): string;
    waterlines(option: string): string;
    export_model(option: string, format: string, below: string): Uint8Array;
  }
  export function engine_version(): string;
  export default function init(input?: unknown): Promise<unknown>;
}
declare module "*.wasm?url" {
  const url: string;
  export default url;
}
