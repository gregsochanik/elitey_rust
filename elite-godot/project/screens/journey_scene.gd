extends Node3D
## The space between the jump and the station: a wireframe ship and a starfield that streaks during a jump.

const Style := preload("res://theme/visual_style.gd")

const PLAYER_MODEL := "cobra-mk-iii"
const ENEMY_MODEL := "raider"
const STAR_COUNT := 260
const FIELD := Vector3(60, 36, 120)
const BOLT_SECONDS := 0.28
const CRUISE_SPEED := 4.0
const JUMP_SPEED := 160.0
const JUMP_SECONDS := 3.0

const PLAYER_ENCOUNTER := Vector3(-3.4, -0.7, 0.0)
const ENEMY_ENCOUNTER := Vector3(4.4, 0.9, -15.0)

var ship: MeshInstance3D
var enemy: MeshInstance3D
var world := WorldEnvironment.new()
var lights := []
var in_encounter := false
var stars := MeshInstance3D.new()
var star_mesh := ImmediateMesh.new()
var star_positions: PackedVector3Array = []
var speed := CRUISE_SPEED
var jump_left := 0.0
var camera: Camera3D
var shake_strength := 0.0
var shake_length := 0.0
var shake_left := 0.0


func _ready() -> void:
	camera = Camera3D.new()
	camera.position = Vector3(0, 1.6, 9)
	camera.rotation_degrees = Vector3(-8, 0, 0)
	add_child(camera)

	world.environment = Style.environment()
	add_child(world)

	ship = Style.model_node(PLAYER_MODEL, "player")
	add_child(ship)

	enemy = Style.model_node(ENEMY_MODEL, "enemy")
	enemy.visible = false
	add_child(enemy)

	stars.mesh = star_mesh
	stars.material_override = Style.material("stars")
	add_child(stars)
	for i in STAR_COUNT:
		star_positions.append(_random_star(true))
	_relight()
	_relight()
	Style.events.changed.connect(_restyle)


func _relight() -> void:
	for light in lights:
		light.queue_free()
	lights = Style.lights()
	for light in lights:
		add_child(light)


func _restyle() -> void:
	world.environment = Style.environment()
	_relight()
	Style.restyle(ship, PLAYER_MODEL, "player")
	Style.restyle(enemy, ENEMY_MODEL, "enemy")
	stars.material_override = Style.material("stars")


func start_jump() -> void:
	jump_left = JUMP_SECONDS


func begin_encounter() -> void:
	in_encounter = true
	ship.position = PLAYER_ENCOUNTER
	enemy.position = ENEMY_ENCOUNTER
	enemy.visible = true
	enemy.scale = Vector3.ONE
	Style.restyle(enemy, ENEMY_MODEL, "enemy")


func end_encounter() -> void:
	in_encounter = false
	ship.position = Vector3.ZERO
	ship.visible = true
	enemy.visible = false


## The enemy breaks off and streaks away.
func enemy_leaves() -> void:
	var tween := create_tween()
	tween.tween_property(enemy, "position", ENEMY_ENCOUNTER + Vector3(10, 4, -70), 0.9)
	tween.tween_callback(func(): enemy.visible = false)


## The enemy gives in: it stops firing and goes pale.
func enemy_yields() -> void:
	enemy.material_override.albedo_color = Color(0.9, 0.9, 0.9)


func point_of(player: bool) -> Vector3:
	return ship.position if player else enemy.position


## A bolt of laser fire from one ship at the other with a muzzle flash; a miss flies wide.
func fire_beam(from_player: bool, hit: bool) -> void:
	var from := point_of(from_player)
	var target := point_of(not from_player)
	if not hit:
		target += Vector3(randf_range(-4.0, 4.0), randf_range(2.0, 4.0), 0.0)
	var role := "beam_player" if from_player else "beam_enemy"
	_burst(from, 9, 0.1, 0.9, role, 0.18)

	var material := Style.material(role, true)
	var bolt := MeshInstance3D.new()
	var box := BoxMesh.new()
	box.size = Vector3(0.09, 0.09, 1.6)
	bolt.mesh = box
	bolt.material_override = material
	bolt.position = from
	add_child(bolt)
	bolt.look_at(target)
	var tween := create_tween()
	tween.tween_property(bolt, "position", target, BOLT_SECONDS)
	tween.tween_callback(_bolt_lands.bind(bolt, target, role, hit, from_player))


func _bolt_lands(bolt: Node3D, at: Vector3, role: String, hit: bool, from_player: bool) -> void:
	bolt.queue_free()
	if hit:
		_burst(at, 22, 0.15, 1.1, role, 0.35)
		shake(0.1, 0.25)


## A ripple spreading over the shield of the ship that was hit, and a faint bubble.
func flash_shield(on_player: bool) -> void:
	var at := point_of(on_player)
	var attacker := point_of(not on_player)
	var bubble_material := Style.material("shield", true)
	bubble_material.albedo_color.a = 0.55
	var bubble := MeshInstance3D.new()
	bubble.mesh = _sphere_mesh(2.6)
	bubble.material_override = bubble_material
	bubble.position = at
	add_child(bubble)
	_fade_and_free(bubble, bubble_material, 0.4, 1.08)

	var ring_material := Style.material("shield", true)
	var ring := MeshInstance3D.new()
	ring.mesh = _ring_mesh(0.5)
	ring.material_override = ring_material
	ring.position = at + (attacker - at).normalized() * 2.4
	add_child(ring)
	ring.look_at(attacker)
	_fade_and_free(ring, ring_material, 0.55, 7.0)


## The ship breaks up: a flash of lines, spinning debris and a screen shake.
func explode(on_player: bool) -> void:
	var at := point_of(on_player)
	_burst(at, 70, 0.3, 2.4, "explosion", 1.1, 5.0)
	_debris(at)
	shake(0.5, 0.7)
	if on_player:
		ship.visible = false
	else:
		enemy.visible = false


## Pieces of hull tumbling away from the wreck.
func _debris(at: Vector3) -> void:
	var material := Style.material("explosion", true)
	for i in 14:
		var piece := MeshInstance3D.new()
		var box := BoxMesh.new()
		box.size = Vector3(randf_range(0.08, 0.3), randf_range(0.05, 0.14), randf_range(0.08, 0.3))
		piece.mesh = box
		piece.material_override = material
		piece.position = at
		add_child(piece)
		var drift := Vector3(randf_range(-1, 1), randf_range(-1, 1), randf_range(-1, 1)).normalized() * randf_range(2.0, 5.5)
		var tween := create_tween()
		tween.set_parallel(true)
		tween.tween_property(piece, "position", at + drift, 1.4).set_ease(Tween.EASE_OUT)
		tween.tween_property(piece, "rotation", Vector3(randf_range(-6, 6), randf_range(-6, 6), randf_range(-6, 6)), 1.4)
		tween.chain().tween_callback(piece.queue_free)
	var fade := create_tween()
	fade.tween_property(material, "albedo_color:a", 0.0, 1.4)


## Short lines flying out from a point: muzzle flashes, hit sparks and the start of an explosion.
func _burst(at: Vector3, count: int, inner: float, outer: float, role: String, seconds: float, grow := 1.8) -> void:
	var material := Style.material(role, true)
	var mesh := ImmediateMesh.new()
	mesh.surface_begin(Mesh.PRIMITIVE_LINES)
	for i in count:
		var direction := Vector3(randf_range(-1, 1), randf_range(-1, 1), randf_range(-1, 1)).normalized()
		mesh.surface_add_vertex(direction * randf_range(inner, inner * 2.5))
		mesh.surface_add_vertex(direction * randf_range(outer * 0.6, outer))
	mesh.surface_end()
	var node := MeshInstance3D.new()
	node.mesh = mesh
	node.material_override = material
	node.position = at
	add_child(node)
	_fade_and_free(node, material, seconds, grow)


## Shake the camera; the strength dies away over `seconds`.
func shake(strength: float, seconds: float) -> void:
	shake_strength = maxf(shake_strength, strength)
	shake_length = maxf(shake_length, seconds)
	shake_left = maxf(shake_left, seconds)


func _ring_mesh(radius: float) -> ImmediateMesh:
	var mesh := ImmediateMesh.new()
	mesh.surface_begin(Mesh.PRIMITIVE_LINES)
	var steps := 40
	for i in steps:
		var a := TAU * i / steps
		var b := TAU * (i + 1) / steps
		mesh.surface_add_vertex(Vector3(cos(a), sin(a), 0) * radius)
		mesh.surface_add_vertex(Vector3(cos(b), sin(b), 0) * radius)
	mesh.surface_end()
	return mesh


func _fade_and_free(node: Node3D, material: StandardMaterial3D, seconds: float, grow: float) -> void:
	var tween := create_tween()
	tween.tween_property(material, "albedo_color:a", 0.0, seconds)
	tween.parallel().tween_property(node, "scale", Vector3.ONE * grow, seconds)
	tween.tween_callback(node.queue_free)


func _sphere_mesh(radius: float) -> ImmediateMesh:
	var mesh := ImmediateMesh.new()
	mesh.surface_begin(Mesh.PRIMITIVE_LINES)
	var rings := 6
	var steps := 24
	for ring in range(1, rings):
		var latitude := PI * ring / rings - PI / 2.0
		for i in steps:
			var a := TAU * i / steps
			var b := TAU * (i + 1) / steps
			mesh.surface_add_vertex(Vector3(cos(latitude) * cos(a), sin(latitude), cos(latitude) * sin(a)) * radius)
			mesh.surface_add_vertex(Vector3(cos(latitude) * cos(b), sin(latitude), cos(latitude) * sin(b)) * radius)
	for meridian in 8:
		var longitude := TAU * meridian / 8.0
		for i in steps:
			var a := TAU * i / steps
			var b := TAU * (i + 1) / steps
			mesh.surface_add_vertex(Vector3(cos(a) * cos(longitude), sin(a), cos(a) * sin(longitude)) * radius)
			mesh.surface_add_vertex(Vector3(cos(b) * cos(longitude), sin(b), cos(b) * sin(longitude)) * radius)
	mesh.surface_end()
	return mesh


func _process(delta: float) -> void:
	_shake_camera(delta)
	_pulse_engines()
	if enemy.visible:
		enemy.rotation.y = sin(Time.get_ticks_msec() / 1400.0) * 0.5 + 0.35
		enemy.position.y = ENEMY_ENCOUNTER.y + sin(Time.get_ticks_msec() / 700.0) * 0.15
	ship.rotation.y += delta * (0.4 if jump_left <= 0.0 and not in_encounter else 0.05)
	ship.rotation.z = sin(Time.get_ticks_msec() / 900.0) * 0.12

	var target := JUMP_SPEED if jump_left > 0.0 else CRUISE_SPEED
	jump_left = maxf(jump_left - delta, 0.0)
	speed = lerpf(speed, target, minf(delta * 3.0, 1.0))

	star_mesh.clear_surfaces()
	star_mesh.surface_begin(Mesh.PRIMITIVE_LINES)
	for i in star_positions.size():
		var star := star_positions[i]
		var tail := star - Vector3(0, 0, maxf(speed * 0.03, 0.04))
		star_mesh.surface_add_vertex(star)
		star_mesh.surface_add_vertex(tail)
		star.z += speed * delta
		if star.z > 12.0:
			star = _random_star(false)
		star_positions[i] = star
	star_mesh.surface_end()


func _random_star(anywhere: bool) -> Vector3:
	var z := randf_range(-FIELD.z, 12.0) if anywhere else -FIELD.z
	var x := randf_range(-FIELD.x, FIELD.x)
	var y := randf_range(-FIELD.y, FIELD.y)
	if absf(x) < 3.0 and absf(y) < 2.0:
		x += 6.0
	return Vector3(x, y, z)


func _shake_camera(delta: float) -> void:
	if shake_left <= 0.0:
		camera.h_offset = 0.0
		camera.v_offset = 0.0
		shake_strength = 0.0
		shake_length = 0.0
		return
	shake_left = maxf(shake_left - delta, 0.0)
	var strength := shake_strength * shake_left / maxf(shake_length, 0.001)
	camera.h_offset = randf_range(-strength, strength)
	camera.v_offset = randf_range(-strength, strength)


func _pulse_engines() -> void:
	var boost := 1.0 + clampf(speed / JUMP_SPEED, 0.0, 1.0) * 1.6
	var pulse := (1.0 + sin(Time.get_ticks_msec() / 70.0) * 0.18) * boost
	for craft in [ship, enemy]:
		for child in craft.get_children():
			if child.is_in_group("engine_glow"):
				child.scale = Vector3.ONE * pulse
