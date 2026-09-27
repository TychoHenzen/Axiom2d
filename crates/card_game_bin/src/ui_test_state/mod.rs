use card_game::card::rendering::art_shader::ShaderVariant;

use super::{
    BoosterOpenPhase, BoosterOpening, BoosterPack, Card, CardLabel, CardReader, CardSignature,
    CardZone, CombinerDevice, ConditionEffect, DORMANT_WGSL, DragState, Entity, FOIL_WGSL, Hand,
    INTENSE_WGSL, Jack, JackSocket, MeshOverlays, MouseButton, MouseState, PathBuf, Query, Res,
    SCREEN_SPLINE_GOLDEN_SIGNATURE, ShapeRepository, ShapeVariant, SignatureSpace, StashGrid,
    StashHoverPreview, StashVisible, Transform2D, UI_TEST_BOOSTER_ENTITY,
    UI_TEST_BOOSTER_SIGNATURE, UI_TEST_CARD_ENTITY, UI_TEST_COMBINER_ENTITY,
    UI_TEST_PLUGIN_WIRING_SCENARIO, UI_TEST_SCENARIO, UI_TEST_SCREEN_ENTITY,
    UI_TEST_SCREEN_JACK_ENTITY, UI_TEST_SECOND_CARD_ENTITY, UI_TEST_STATE_FILE, UiTestCableParams,
    UiTestRenderingParams, Vec2, ZoneConfig, compute_seed, format_geometry, format_terrain_corners,
    nearest_rendered_point, rendered_centerline, rendered_max_deviation,
    screen_spline_golden_points, select_art_for_signature,
};

struct SecondaryCardState {
    entity: Entity,
    signature: CardSignature,
    zone: String,
    position: Vec2,
    reader_loaded: bool,
}

#[allow(clippy::struct_excessive_bools)]
struct CardState {
    entity: Entity,
    signature: CardSignature,
    face_up: bool,
    zone: CardZone,
    position: Vec2,
    rotation: f32,
    second: Option<SecondaryCardState>,
    dragging: bool,
    second_dragging: bool,
    stash_origin: String,
    stash_cursor_follow: bool,
    stash_slot: String,
    stash_slot_present: bool,
    stash_hovered: bool,
    hand_contains: bool,
    hand_count: usize,
    holder: String,
    holder_occupied: bool,
    zone_has_physics: bool,
    zone_render_layer: String,
    zone_has_item_form: bool,
}

struct ReaderState {
    present: bool,
    loaded: bool,
    signature: String,
    space_contains: bool,
    space_source_count: usize,
    space_radius: f32,
    feedback: &'static str,
}

struct ScreenState {
    present: bool,
    signature: String,
    geometry_match: bool,
    geometry_max_error: f32,
    expected_geometry: [String; 4],
    observed_geometry: [String; 4],
}

#[allow(clippy::struct_excessive_bools)]
struct CombinerState {
    present: bool,
    input_a_connected: bool,
    input_b_connected: bool,
    input_a_source_count: usize,
    input_b_source_count: usize,
    output_source_count: usize,
    output_control_points: usize,
    output_radius: f32,
    output_contains_primary: bool,
    output_contains_secondary: bool,
}

struct CableState {
    present: bool,
    connected: bool,
    anchor_count: usize,
    anchor_0: Vec2,
    source: Vec2,
    dest: Vec2,
    rendered_min: Vec2,
    rendered_max: Vec2,
    rendered_vertex_count: usize,
    rendered_source: Vec2,
    rendered_anchor: Vec2,
    rendered_dest: Vec2,
    rendered_max_deviation: f32,
    pending_dragging: bool,
}

struct IdentityState {
    signature: String,
    second_signature: String,
    seed: u64,
    rarity: String,
    tier: String,
    name: String,
    art_signature: String,
    art_element: String,
    art_aspect: String,
    art_shape_count: usize,
}

#[allow(clippy::struct_excessive_bools)]
struct BoosterState {
    dragging: bool,
    pack_present: bool,
    phase: String,
    card_count: usize,
    rendered_position: Vec2,
    expected_card_seed: u64,
    opened_present: bool,
    opened_zone: String,
    opened_seed: u64,
    opened_face_up: bool,
    opened_position: Vec2,
}

#[allow(clippy::struct_excessive_bools)]
struct ShaderState {
    variant: String,
    condition: String,
    variant_source_length: usize,
    variant_source_matches: bool,
    variant_overlay_present: bool,
    variant_overlay_visible: bool,
    variant_overlay_handle: u32,
    variant_overlay_handle_matches: bool,
    variant_overlay_vertex_count: usize,
    condition_tier: &'static str,
    condition_source_length: usize,
    condition_source_matches: bool,
    condition_overlay_present: bool,
    condition_overlay_visible: bool,
    condition_overlay_handle: u32,
    condition_overlay_handle_matches: bool,
    condition_overlay_front_only: bool,
    condition_overlay_vertex_count: usize,
    overlay_count: usize,
}

struct TerrainState {
    active: bool,
    grid_width: usize,
    grid_height: usize,
    visual_tile_count: usize,
    rendered_tile_count: usize,
    cell_1_1: i32,
    visual_tile_7_corners: String,
    visual_tile_7_seed: u32,
    click_count: u32,
    last_clicked_tile: i32,
}

#[allow(clippy::struct_excessive_bools)]
struct Snapshot {
    scenario: String,
    card: CardState,
    reader: ReaderState,
    screen: ScreenState,
    combiner: CombinerState,
    cable: CableState,
    identity: IdentityState,
    booster: BoosterState,
    shader: ShaderState,
    terrain: TerrainState,
    plugin_wiring_fixture_ready: bool,
    mouse_position: Vec2,
    left_pressed: bool,
    right_pressed: bool,
    stash_visible: bool,
    stash_page: u8,
    stash_storage_page: i32,
}

fn format_signature(signature: &CardSignature) -> String {
    signature
        .axes()
        .iter()
        .map(|axis| format!("{axis:.6}"))
        .collect::<Vec<_>>()
        .join(",")
}

fn collect_card_state(
    drag_state: &DragState,
    hand: &Hand,
    stash_grid: &StashGrid,
    stash_hover: &StashHoverPreview,
    readers: &Query<&CardReader>,
    rendering: &UiTestRenderingParams,
) -> Option<CardState> {
    let entity = UI_TEST_CARD_ENTITY.get().copied()?;
    let Ok((_, card, zone, transform, _)) = rendering.cards.get(entity) else {
        return None;
    };
    let second = UI_TEST_SECOND_CARD_ENTITY
        .get()
        .and_then(|second_entity| rendering.cards.get(*second_entity).ok())
        .map(
            |(second_entity, second_card, second_zone, second_transform, _)| SecondaryCardState {
                entity: second_entity,
                signature: second_card.signature,
                zone: format!("{second_zone:?}"),
                position: second_transform.position,
                reader_loaded: readers
                    .iter()
                    .any(|reader| reader.loaded == Some(second_entity)),
            },
        );
    let dragging = drag_state
        .dragging
        .as_ref()
        .is_some_and(|drag| drag.entity == entity);
    let second_dragging = second.as_ref().is_some_and(|second| {
        drag_state
            .dragging
            .as_ref()
            .is_some_and(|drag| drag.entity == second.entity)
    });
    let stash_cursor_follow = drag_state
        .dragging
        .as_ref()
        .is_some_and(|drag| drag.entity == entity && drag.stash_cursor_follow);
    let stash_origin = drag_state
        .dragging
        .as_ref()
        .and_then(|drag| match drag.origin_zone {
            CardZone::Stash { page, col, row } => Some(format!("{page}:{col}:{row}")),
            _ => None,
        })
        .unwrap_or_else(|| "none".to_owned());
    let stash_slot = match zone {
        CardZone::Stash { page, col, row } => format!("{page}:{col}:{row}"),
        _ => "none".to_owned(),
    };
    let stash_slot_present = stash_grid
        .current_storage_page()
        .is_some_and(|page| stash_grid.get(page, 0, 0).is_some());
    let hand_contains = hand.cards().contains(&entity);
    let holder = match zone {
        CardZone::Hand(index) => format!("Hand({index})"),
        _ => "none".to_owned(),
    };
    let zone_config = ZoneConfig::for_zone(zone);
    Some(CardState {
        entity,
        signature: card.signature,
        face_up: card.face_up,
        zone: *zone,
        position: transform.position,
        rotation: transform.rotation,
        second,
        dragging,
        second_dragging,
        stash_origin,
        stash_cursor_follow,
        stash_slot,
        stash_slot_present,
        stash_hovered: stash_hover.hovered_entity == Some(entity),
        hand_contains,
        hand_count: hand.len(),
        holder,
        holder_occupied: matches!(zone, CardZone::Hand(_)) && hand_contains,
        zone_has_physics: zone_config.has_physics,
        zone_render_layer: format!("{:?}", zone_config.render_layer),
        zone_has_item_form: zone_config.has_item_form,
    })
}

fn collect_reader_state(
    card: &CardState,
    readers: &Query<&CardReader>,
    reader_jacks: &Query<&Jack<SignatureSpace>>,
) -> ReaderState {
    let reader = readers
        .iter()
        .find(|reader| reader.loaded == Some(card.entity));
    let reader_space = reader
        .and_then(|reader| reader_jacks.get(reader.jack_entity).ok())
        .and_then(|jack| jack.data.as_ref());
    let loaded = reader.is_some();
    ReaderState {
        present: readers.iter().next().is_some(),
        loaded,
        signature: reader_space
            .and_then(|space| space.control_points.first())
            .map_or_else(String::new, format_signature),
        space_contains: reader_space.is_some_and(|space| space.contains(&card.signature)),
        space_source_count: reader_space.map_or(0, |space| space.source_cards.len()),
        space_radius: reader_space.map_or(0.0, |space| space.radius),
        feedback: if loaded { "lit" } else { "dim" },
    }
}

fn collect_screen_state(
    cable_params: &UiTestCableParams,
    reader_jacks: &Query<&Jack<SignatureSpace>>,
) -> ScreenState {
    let screen_space = UI_TEST_SCREEN_JACK_ENTITY
        .get()
        .and_then(|screen_jack| reader_jacks.get(*screen_jack).ok())
        .and_then(|jack| jack.data.as_ref());
    let signature = screen_space
        .and_then(|space| space.control_points.first())
        .map_or_else(String::new, format_signature);
    let screen_entity = UI_TEST_SCREEN_ENTITY.get().copied();
    let mut expected_geometry = std::array::from_fn(|_| String::new());
    let mut observed_geometry = std::array::from_fn(|_| String::new());
    let mut geometry_max_error = -1.0;
    let mut geometry_match = false;
    if UI_TEST_SCENARIO
        .get()
        .is_some_and(|scenario| scenario == "seeded-card-screen-spline")
    {
        let expected = screen_spline_golden_points();
        for (display_index, points) in expected.iter().enumerate() {
            expected_geometry[display_index] = format_geometry(points);
        }
        if let (Some(_), Some(screen_entity)) = (screen_space, screen_entity) {
            let mut observed_points = Vec::with_capacity(expected.len());
            for (display_index, observed) in observed_geometry.iter_mut().enumerate() {
                let points = cable_params
                    .screen_shapes
                    .iter()
                    .find(|(signal, parent, _, visible)| {
                        signal.display_index() == Some(display_index)
                            && parent.0 == screen_entity
                            && visible.0
                    })
                    .and_then(|(_, _, shape, _)| match &shape.variant {
                        ShapeVariant::Polygon { points } => Some(points.clone()),
                        _ => None,
                    })
                    .unwrap_or_default();
                *observed = format_geometry(&points);
                observed_points.push(points);
            }
            if expected
                .iter()
                .zip(&observed_points)
                .all(|(expected, observed)| expected.len() == observed.len())
            {
                geometry_max_error = expected
                    .iter()
                    .zip(&observed_points)
                    .flat_map(|(expected, observed)| {
                        expected
                            .iter()
                            .zip(observed)
                            .map(|(expected, observed)| (*expected - *observed).length())
                    })
                    .fold(0.0, f32::max);
                geometry_match =
                    signature == SCREEN_SPLINE_GOLDEN_SIGNATURE && geometry_max_error <= 0.01;
            }
        }
    }
    ScreenState {
        present: screen_entity.is_some(),
        signature,
        geometry_match,
        geometry_max_error,
        expected_geometry,
        observed_geometry,
    }
}

fn cable_is_connected(
    cable_params: &UiTestCableParams,
    sockets: &Query<&JackSocket>,
    socket_entity: Entity,
) -> bool {
    sockets.get(socket_entity).is_ok_and(|socket| {
        socket.connected_cable.is_some_and(|cable_entity| {
            cable_params
                .cables
                .get(cable_entity)
                .is_ok_and(|(_, cable, _, _, _)| cable.is_some())
        })
    })
}

fn collect_combiner_state(
    card: &CardState,
    cable_params: &UiTestCableParams,
    combiners: &Query<&CombinerDevice>,
    reader_jacks: &Query<&Jack<SignatureSpace>>,
    sockets: &Query<&JackSocket>,
) -> CombinerState {
    let combiner = UI_TEST_COMBINER_ENTITY
        .get()
        .and_then(|combiner_entity| combiners.get(*combiner_entity).ok());
    let input_a_connected =
        combiner.is_some_and(|device| cable_is_connected(cable_params, sockets, device.input_a));
    let input_b_connected =
        combiner.is_some_and(|device| cable_is_connected(cable_params, sockets, device.input_b));
    let input_a_space = combiner
        .and_then(|device| reader_jacks.get(device.input_a).ok())
        .and_then(|jack| jack.data.as_ref());
    let input_b_space = combiner
        .and_then(|device| reader_jacks.get(device.input_b).ok())
        .and_then(|jack| jack.data.as_ref());
    let output_space = combiner
        .and_then(|device| reader_jacks.get(device.output).ok())
        .and_then(|jack| jack.data.as_ref());
    CombinerState {
        present: combiner.is_some(),
        input_a_connected,
        input_b_connected,
        input_a_source_count: input_a_space.map_or(0, |space| space.source_cards.len()),
        input_b_source_count: input_b_space.map_or(0, |space| space.source_cards.len()),
        output_source_count: output_space.map_or(0, |space| space.source_cards.len()),
        output_control_points: output_space.map_or(0, |space| space.control_points.len()),
        output_radius: output_space.map_or(0.0, |space| space.radius),
        output_contains_primary: output_space.is_some_and(|space| space.contains(&card.signature)),
        output_contains_secondary: card.second.as_ref().is_some_and(|second| {
            output_space.is_some_and(|space| space.contains(&second.signature))
        }),
    }
}

fn collect_cable_state(
    cable_params: &UiTestCableParams,
    sockets: &Query<&JackSocket>,
) -> CableState {
    let mut state = CableState {
        present: false,
        connected: false,
        anchor_count: 0,
        anchor_0: Vec2::ZERO,
        source: Vec2::ZERO,
        dest: Vec2::ZERO,
        rendered_min: Vec2::ZERO,
        rendered_max: Vec2::ZERO,
        rendered_vertex_count: 0,
        rendered_source: Vec2::ZERO,
        rendered_anchor: Vec2::ZERO,
        rendered_dest: Vec2::ZERO,
        rendered_max_deviation: 0.0,
        pending_dragging: cable_params.pending_cable.free_end.is_some(),
    };
    if let Some((_, cable, endpoints, wrap, shape)) = cable_params.cables.iter().next() {
        state.present = true;
        state.connected = cable.is_some_and(|cable| {
            sockets.get(cable.source).is_ok() && sockets.get(cable.dest).is_ok()
        });
        state.anchor_count = wrap.anchors.len();
        state.anchor_0 = wrap
            .anchors
            .first()
            .map_or(Vec2::ZERO, |anchor| anchor.position);
        state.source = cable_params
            .transforms
            .get(endpoints.source)
            .map_or(Vec2::ZERO, |transform| transform.position);
        state.dest = cable_params
            .transforms
            .get(endpoints.dest)
            .map_or(Vec2::ZERO, |transform| transform.position);
        if let ShapeVariant::Polygon { points } = &shape.variant {
            state.rendered_vertex_count = points.len();
            if let Some(first) = points.first() {
                state.rendered_min = *first;
                state.rendered_max = *first;
                for point in points.iter().skip(1) {
                    state.rendered_min = Vec2::new(
                        state.rendered_min.x.min(point.x),
                        state.rendered_min.y.min(point.y),
                    );
                    state.rendered_max = Vec2::new(
                        state.rendered_max.x.max(point.x),
                        state.rendered_max.y.max(point.y),
                    );
                }
            }
            let centerline = rendered_centerline(points);
            state.rendered_source = centerline.first().copied().unwrap_or(Vec2::ZERO);
            state.rendered_dest = centerline.last().copied().unwrap_or(Vec2::ZERO);
            state.rendered_anchor = wrap.anchors.first().map_or(Vec2::ZERO, |anchor| {
                nearest_rendered_point(&centerline, anchor.position)
            });
            state.rendered_max_deviation =
                rendered_max_deviation(&centerline, state.rendered_source, state.rendered_dest);
        }
    }
    state
}

fn collect_identity_state(
    card: &CardState,
    card_labels: &Query<&CardLabel>,
    art_repository: &ShapeRepository,
) -> IdentityState {
    let selected_art = select_art_for_signature(&card.signature, art_repository);
    IdentityState {
        signature: format_signature(&card.signature),
        second_signature: card
            .second
            .as_ref()
            .map_or_else(String::new, |second| format_signature(&second.signature)),
        seed: compute_seed(&card.signature),
        rarity: format!("{:?}", card.signature.rarity()),
        tier: format!("{:?}", card.signature.card_tier()),
        name: card_labels
            .get(card.entity)
            .map_or_else(|_| String::new(), |label| label.name.clone()),
        art_signature: selected_art
            .map_or_else(String::new, |entry| format_signature(&entry.signature())),
        art_element: selected_art
            .map_or_else(String::new, |entry| format!("{:?}", entry.element())),
        art_aspect: selected_art.map_or_else(String::new, |entry| format!("{:?}", entry.aspect())),
        art_shape_count: selected_art.map_or(0, |entry| entry.shapes().len()),
    }
}

fn collect_booster_state(
    card: &CardState,
    drag_state: &DragState,
    booster_opening: Option<&BoosterOpening>,
    booster_packs: &Query<(Entity, &BoosterPack, &Transform2D)>,
    cards: &Query<(
        Entity,
        &Card,
        &CardZone,
        &Transform2D,
        Option<&MeshOverlays>,
    )>,
) -> BoosterState {
    let booster_pack = UI_TEST_BOOSTER_ENTITY.get().and_then(|expected| {
        booster_packs
            .iter()
            .find(|(candidate, _, _)| *candidate == *expected)
    });
    let expected_signature = UI_TEST_BOOSTER_SIGNATURE.get().copied();
    let opened_card = expected_signature.and_then(|signature| {
        cards.iter().find(|(candidate, candidate_card, _, _, _)| {
            *candidate != card.entity && candidate_card.signature == signature
        })
    });
    let opened_present = opened_card.is_some();
    let opening_phase = booster_opening.map(|opening| match &opening.phase {
        BoosterOpenPhase::MovingToCenter { .. } => "moving_to_center",
        BoosterOpenPhase::Ripping { .. } => "ripping",
        BoosterOpenPhase::LoweringPack { .. } => "lowering_pack",
        BoosterOpenPhase::RevealingCards { .. } => "revealing_cards",
        BoosterOpenPhase::Completing { .. } => "completing",
        BoosterOpenPhase::Done => "done",
    });
    let pack_present = booster_pack.is_some();
    BoosterState {
        dragging: drag_state.dragging.as_ref().is_some_and(|drag| {
            UI_TEST_BOOSTER_ENTITY
                .get()
                .is_some_and(|booster_entity| drag.entity == *booster_entity)
        }),
        pack_present,
        phase: opening_phase
            .unwrap_or(if pack_present {
                "sealed"
            } else if opened_present {
                "done"
            } else {
                "none"
            })
            .to_owned(),
        card_count: booster_opening.map_or_else(
            || booster_pack.map_or(usize::from(opened_present), |(_, pack, _)| pack.cards.len()),
            |opening| opening.cards.len(),
        ),
        rendered_position: booster_pack.map_or(Vec2::ZERO, |(_, _, transform)| transform.position),
        expected_card_seed: expected_signature.map_or(0, |signature| compute_seed(&signature)),
        opened_present,
        opened_zone: opened_card.as_ref().map_or_else(
            || "none".to_owned(),
            |(_, _, zone, _, _)| format!("{zone:?}"),
        ),
        opened_seed: opened_card
            .as_ref()
            .map_or(0, |(_, candidate_card, _, _, _)| {
                compute_seed(&candidate_card.signature)
            }),
        opened_face_up: opened_card
            .as_ref()
            .is_some_and(|(_, candidate_card, _, _, _)| candidate_card.face_up),
        opened_position: opened_card
            .as_ref()
            .map_or(Vec2::ZERO, |(_, _, _, transform, _)| transform.position),
    }
}

fn collect_shader_state(
    card: &CardState,
    overlays: Option<&MeshOverlays>,
    rendering: &UiTestRenderingParams,
) -> ShaderState {
    let shader_variant = ShaderVariant::from_rarity(card.signature.rarity());
    let condition_effect = ConditionEffect::from_tier(card.signature.card_tier());
    let expected_variant_shader = match shader_variant {
        ShaderVariant::None => None,
        ShaderVariant::Embossed => Some(rendering.variant_shaders.embossed),
        ShaderVariant::Glow => Some(rendering.variant_shaders.glow),
        ShaderVariant::Glossy => Some(rendering.variant_shaders.glossy),
        ShaderVariant::Foil => Some(rendering.variant_shaders.foil),
    };
    let expected_condition_shader = match condition_effect {
        ConditionEffect::None => None,
        ConditionEffect::Worn => Some(rendering.tier_shaders.dormant),
        ConditionEffect::Shiny => Some(rendering.tier_shaders.intense),
    };
    let expected_condition_source = match condition_effect {
        ConditionEffect::None => None,
        ConditionEffect::Worn => Some(DORMANT_WGSL),
        ConditionEffect::Shiny => Some(INTENSE_WGSL),
    };
    let (variant_overlay, condition_overlay) = overlays.map_or((None, None), |overlays| {
        let mut variant_overlay = None;
        let mut condition_overlay = None;
        for entry in &overlays.0 {
            if variant_overlay.is_none()
                && expected_variant_shader.is_some_and(|expected| entry.material.shader == expected)
            {
                variant_overlay = Some(entry);
            }
            if condition_overlay.is_none()
                && expected_condition_shader
                    .is_some_and(|expected| entry.material.shader == expected)
            {
                condition_overlay = Some(entry);
            }
        }
        (variant_overlay, condition_overlay)
    });
    ShaderState {
        variant: format!("{shader_variant:?}"),
        condition: format!("{condition_effect:?}"),
        variant_source_length: expected_variant_shader
            .and_then(|shader| rendering.shader_registry.lookup(shader))
            .map_or(0, str::len),
        variant_source_matches: expected_variant_shader
            .is_some_and(|shader| rendering.shader_registry.lookup(shader) == Some(FOIL_WGSL)),
        variant_overlay_present: variant_overlay.is_some(),
        variant_overlay_visible: variant_overlay.is_some_and(|entry| entry.visible),
        variant_overlay_handle: variant_overlay.map_or(0, |entry| entry.material.shader.0),
        variant_overlay_handle_matches: expected_variant_shader.is_none()
            || variant_overlay
                .is_some_and(|entry| Some(entry.material.shader) == expected_variant_shader),
        variant_overlay_vertex_count: variant_overlay.map_or(0, |entry| entry.mesh.vertices.len()),
        condition_tier: match condition_effect {
            ConditionEffect::None => "None",
            ConditionEffect::Worn => "Dormant",
            ConditionEffect::Shiny => "Intense",
        },
        condition_source_length: expected_condition_shader
            .and_then(|shader| rendering.shader_registry.lookup(shader))
            .map_or(0, str::len),
        condition_source_matches: expected_condition_shader
            .zip(expected_condition_source)
            .is_some_and(|(shader, source)| {
                rendering.shader_registry.lookup(shader) == Some(source)
            }),
        condition_overlay_present: condition_overlay.is_some(),
        condition_overlay_visible: condition_overlay.is_some_and(|entry| entry.visible),
        condition_overlay_handle: condition_overlay.map_or(0, |entry| entry.material.shader.0),
        condition_overlay_handle_matches: expected_condition_shader.is_none()
            || condition_overlay
                .is_some_and(|entry| Some(entry.material.shader) == expected_condition_shader),
        condition_overlay_front_only: condition_overlay.is_some_and(|entry| entry.front_only),
        condition_overlay_vertex_count: condition_overlay
            .map_or(0, |entry| entry.mesh.vertices.len()),
        overlay_count: overlays.map_or(0, |overlays| overlays.0.len()),
    }
}

fn collect_terrain_state(rendering: &UiTestRenderingParams) -> TerrainState {
    let visual_tiles = rendering
        .terrain
        .as_ref()
        .map_or_else(Vec::new, |terrain| terrain.grid.visual_tiles());
    let visual_tile_7 = visual_tiles.get(7);
    let terrain = rendering.terrain.as_ref();
    TerrainState {
        active: terrain.is_some(),
        grid_width: terrain.map_or(0, |terrain| terrain.grid.width()),
        grid_height: terrain.map_or(0, |terrain| terrain.grid.height()),
        visual_tile_count: visual_tiles.len(),
        rendered_tile_count: rendering.tiles.iter().count(),
        cell_1_1: terrain
            .and_then(|terrain| terrain.grid.get(1, 1))
            .map_or(-1, |id| i32::from(id.0)),
        visual_tile_7_corners: visual_tile_7
            .map_or_else(String::new, |tile| format_terrain_corners(tile.corners)),
        visual_tile_7_seed: visual_tile_7.map_or(0, |tile| tile.seed),
        click_count: terrain.map_or(0, |terrain| terrain.click_count),
        last_clicked_tile: terrain
            .and_then(|terrain| terrain.last_clicked_tile)
            .map_or(-1, |index| index as i32),
    }
}

fn write_snapshot(snapshot: &Snapshot, state_file: &PathBuf) {
    let card = &snapshot.card;
    let second = card.second.as_ref();
    let reader = &snapshot.reader;
    let screen = &snapshot.screen;
    let combiner = &snapshot.combiner;
    let identity = &snapshot.identity;
    let booster = &snapshot.booster;
    let shader = &snapshot.shader;
    let cable = &snapshot.cable;
    let terrain = &snapshot.terrain;
    let snapshot_text = format!(
        "scenario={}\ndragging={}\nsecond_dragging={}\nsecond_zone={}\nsecond_rendered_x={:.1}\nsecond_rendered_y={:.1}\nsecond_reader_loaded={}\nbooster_dragging={}\nzone={:?}\nhand_contains={}\nhand_count={}\nstash_visible={}\nstash_page={}\nstash_storage_page={}\nstash_slot={}\nstash_slot_present={}\nstash_origin={}\nstash_cursor_follow={}\nstash_hovered={}\nreader_loaded={}\nreader_signature={}\nreader_space_contains={}\nreader_space_source_count={}\nreader_space_radius={:.4}\nreader_feedback={}\nscreen_signature={}\nscreen_geometry_tolerance=0.01\nscreen_geometry_match={}\nscreen_geometry_max_error={:.6}\nscreen_geometry_panel_count=4\nscreen_expected_geometry_0={}\nscreen_expected_geometry_1={}\nscreen_expected_geometry_2={}\nscreen_expected_geometry_3={}\nscreen_observed_geometry_0={}\nscreen_observed_geometry_1={}\nscreen_observed_geometry_2={}\nscreen_observed_geometry_3={}\ncombiner_input_a_connected={}\ncombiner_input_b_connected={}\ncombiner_input_a_source_count={}\ncombiner_input_b_source_count={}\ncombiner_output_source_count={}\ncombiner_output_control_points={}\ncombiner_output_radius={:.4}\ncombiner_output_contains_primary={}\ncombiner_output_contains_secondary={}\nidentity_signature={}\nsecond_identity_signature={}\nidentity_seed={}\nidentity_rarity={}\nidentity_tier={}\nidentity_name={}\nart_signature={}\nart_element={}\nart_aspect={}\nart_shape_count={}\nbooster_pack_present={}\nbooster_phase={}\nbooster_card_count={}\nbooster_rendered_x={:.1}\nbooster_rendered_y={:.1}\nexpected_card_seed={}\nopened_card_present={}\nopened_card_zone={}\nopened_card_seed={}\nopened_card_face_up={}\nopened_card_x={:.1}\nopened_card_y={:.1}\nrendered_x={:.1}\nrendered_y={:.1}\nrotation={:.4}\nface_up={}\nmouse_x={:.1}\nmouse_y={:.1}\nleft_pressed={}\nright_pressed={}\nholder={}\nholder_occupied={}\nzone_has_physics={}\nzone_render_layer={}\nzone_has_item_form={}\n",
        snapshot.scenario,
        card.dragging,
        card.second_dragging,
        second.map_or("none", |second| second.zone.as_str()),
        second.map_or(0.0, |second| second.position.x),
        second.map_or(0.0, |second| second.position.y),
        second.is_some_and(|second| second.reader_loaded),
        booster.dragging,
        card.zone,
        card.hand_contains,
        card.hand_count,
        snapshot.stash_visible,
        snapshot.stash_page,
        snapshot.stash_storage_page,
        card.stash_slot,
        card.stash_slot_present,
        card.stash_origin,
        card.stash_cursor_follow,
        card.stash_hovered,
        reader.loaded,
        reader.signature,
        reader.space_contains,
        reader.space_source_count,
        reader.space_radius,
        reader.feedback,
        screen.signature,
        screen.geometry_match,
        screen.geometry_max_error,
        screen.expected_geometry[0],
        screen.expected_geometry[1],
        screen.expected_geometry[2],
        screen.expected_geometry[3],
        screen.observed_geometry[0],
        screen.observed_geometry[1],
        screen.observed_geometry[2],
        screen.observed_geometry[3],
        combiner.input_a_connected,
        combiner.input_b_connected,
        combiner.input_a_source_count,
        combiner.input_b_source_count,
        combiner.output_source_count,
        combiner.output_control_points,
        combiner.output_radius,
        combiner.output_contains_primary,
        combiner.output_contains_secondary,
        identity.signature,
        identity.second_signature,
        identity.seed,
        identity.rarity,
        identity.tier,
        identity.name,
        identity.art_signature,
        identity.art_element,
        identity.art_aspect,
        identity.art_shape_count,
        booster.pack_present,
        booster.phase,
        booster.card_count,
        booster.rendered_position.x,
        booster.rendered_position.y,
        booster.expected_card_seed,
        booster.opened_present,
        booster.opened_zone,
        booster.opened_seed,
        booster.opened_face_up,
        booster.opened_position.x,
        booster.opened_position.y,
        card.position.x,
        card.position.y,
        card.rotation,
        card.face_up,
        snapshot.mouse_position.x,
        snapshot.mouse_position.y,
        snapshot.left_pressed,
        snapshot.right_pressed,
        card.holder,
        card.holder_occupied,
        card.zone_has_physics,
        card.zone_render_layer,
        card.zone_has_item_form,
    );
    let snapshot_text = format!(
        "{snapshot_text}reader_present={}\nscreen_present={}\ncombiner_present={}\nplugin_wiring_fixture_ready={}\n",
        reader.present, screen.present, combiner.present, snapshot.plugin_wiring_fixture_ready,
    );
    let snapshot_text = format!(
        "{snapshot_text}shader_variant={}\ncondition_effect={}\nvariant_shader_source_length={}\nvariant_shader_source_matches={}\nvariant_overlay_present={}\nvariant_overlay_visible={}\nvariant_overlay_handle={}\nvariant_overlay_handle_matches={}\nvariant_overlay_vertex_count={}\ncondition_overlay_tier={}\ncondition_overlay_source_length={}\ncondition_overlay_source_matches={}\ncondition_overlay_present={}\ncondition_overlay_visible={}\ncondition_overlay_handle={}\ncondition_overlay_handle_matches={}\ncondition_overlay_front_only={}\ncondition_overlay_vertex_count={}\noverlay_count={}\n",
        shader.variant,
        shader.condition,
        shader.variant_source_length,
        shader.variant_source_matches,
        shader.variant_overlay_present,
        shader.variant_overlay_visible,
        shader.variant_overlay_handle,
        shader.variant_overlay_handle_matches,
        shader.variant_overlay_vertex_count,
        shader.condition_tier,
        shader.condition_source_length,
        shader.condition_source_matches,
        shader.condition_overlay_present,
        shader.condition_overlay_visible,
        shader.condition_overlay_handle,
        shader.condition_overlay_handle_matches,
        shader.condition_overlay_front_only,
        shader.condition_overlay_vertex_count,
        shader.overlay_count,
    );
    let snapshot_text = format!(
        "{snapshot_text}pending_cable_dragging={}\ncable_present={}\ncable_connected={}\ncable_anchor_count={}\ncable_anchor_0_x={:.1}\ncable_anchor_0_y={:.1}\ncable_source_x={:.1}\ncable_source_y={:.1}\ncable_dest_x={:.1}\ncable_dest_y={:.1}\ncable_rendered_source_x={:.1}\ncable_rendered_source_y={:.1}\ncable_rendered_anchor_x={:.1}\ncable_rendered_anchor_y={:.1}\ncable_rendered_dest_x={:.1}\ncable_rendered_dest_y={:.1}\ncable_rendered_max_deviation={:.1}\ncable_rendered_min_x={:.1}\ncable_rendered_min_y={:.1}\ncable_rendered_max_x={:.1}\ncable_rendered_max_y={:.1}\ncable_rendered_vertex_count={}\nterrain_active={}\nterrain_grid_width={}\nterrain_grid_height={}\nterrain_visual_tile_count={}\nterrain_rendered_tile_count={}\nterrain_cell_1_1={}\nterrain_visual_tile_7_corners={}\nterrain_visual_tile_7_seed={}\nterrain_click_count={}\nterrain_last_clicked_tile={}\n",
        cable.pending_dragging,
        cable.present,
        cable.connected,
        cable.anchor_count,
        cable.anchor_0.x,
        cable.anchor_0.y,
        cable.source.x,
        cable.source.y,
        cable.dest.x,
        cable.dest.y,
        cable.rendered_source.x,
        cable.rendered_source.y,
        cable.rendered_anchor.x,
        cable.rendered_anchor.y,
        cable.rendered_dest.x,
        cable.rendered_dest.y,
        cable.rendered_max_deviation,
        cable.rendered_min.x,
        cable.rendered_min.y,
        cable.rendered_max.x,
        cable.rendered_max.y,
        cable.rendered_vertex_count,
        terrain.active,
        terrain.grid_width,
        terrain.grid_height,
        terrain.visual_tile_count,
        terrain.rendered_tile_count,
        terrain.cell_1_1,
        terrain.visual_tile_7_corners,
        terrain.visual_tile_7_seed,
        terrain.click_count,
        terrain.last_clicked_tile,
    );
    std::fs::write(state_file, snapshot_text).expect("failed to write UI test state snapshot");
}

pub(super) fn collect_and_write(
    drag_state: Res<DragState>,
    hand: Res<Hand>,
    mouse: Res<MouseState>,
    cable_params: UiTestCableParams,
    stash_grid: Res<StashGrid>,
    stash_hover: Res<StashHoverPreview>,
    stash_visible: Res<StashVisible>,
    art_repository: Res<ShapeRepository>,
    booster_opening: Option<Res<BoosterOpening>>,
    card_labels: Query<&CardLabel>,
    booster_packs: Query<(Entity, &BoosterPack, &Transform2D)>,
    readers: Query<&CardReader>,
    reader_jacks: Query<&Jack<SignatureSpace>>,
    combiners: Query<&CombinerDevice>,
    sockets: Query<&JackSocket>,
    rendering: UiTestRenderingParams,
) {
    let Some(card) = collect_card_state(
        &drag_state,
        &hand,
        &stash_grid,
        &stash_hover,
        &readers,
        &rendering,
    ) else {
        return;
    };
    let reader = collect_reader_state(&card, &readers, &reader_jacks);
    let screen = collect_screen_state(&cable_params, &reader_jacks);
    let combiner =
        collect_combiner_state(&card, &cable_params, &combiners, &reader_jacks, &sockets);
    let cable = collect_cable_state(&cable_params, &sockets);
    let identity = collect_identity_state(&card, &card_labels, &art_repository);
    let booster = collect_booster_state(
        &card,
        &drag_state,
        booster_opening.as_deref(),
        &booster_packs,
        &rendering.cards,
    );
    let overlays = rendering
        .cards
        .get(card.entity)
        .ok()
        .and_then(|(_, _, _, _, overlays)| overlays);
    let shader = collect_shader_state(&card, overlays, &rendering);
    let terrain = collect_terrain_state(&rendering);
    let scenario = UI_TEST_SCENARIO
        .get()
        .expect("UI test scenario must be configured before the app runs")
        .clone();
    let plugin_wiring_fixture_ready = scenario == UI_TEST_PLUGIN_WIRING_SCENARIO
        && reader.present
        && screen.present
        && combiner.present
        && booster.pack_present
        && terrain.active;
    let snapshot = Snapshot {
        scenario,
        card,
        reader,
        screen,
        combiner,
        cable,
        identity,
        booster,
        shader,
        terrain,
        plugin_wiring_fixture_ready,
        mouse_position: mouse.screen_pos(),
        left_pressed: mouse.pressed(MouseButton::Left),
        right_pressed: mouse.pressed(MouseButton::Right),
        stash_visible: stash_visible.0,
        stash_page: stash_grid.current_page(),
        stash_storage_page: stash_grid.current_storage_page().map_or(-1, i32::from),
    };
    let state_file = UI_TEST_STATE_FILE
        .get()
        .expect("UI test state file must be configured before the app runs");
    write_snapshot(&snapshot, state_file);
}
