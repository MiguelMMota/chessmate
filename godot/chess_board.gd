extends Control

# Debug utilities
const DebugUtils = preload("res://debug_utils.gd")

# Board visual properties
const BOARD_SIZE = 640
const SQUARE_SIZE = BOARD_SIZE / 8
const LIGHT_SQUARE_COLOR = Color(0.93, 0.85, 0.71)
const DARK_SQUARE_COLOR = Color(0.72, 0.53, 0.38)
const SELECTED_COLOR = Color(0.8, 0.8, 0.3, 0.5)
const LEGAL_MOVE_COLOR = Color(0.3, 0.8, 0.3, 0.5)

# Piece colors
const PEARL_WHITE_COLOR = Color(0.97, 0.96, 0.94)
const BLACK_COLOR = Color(0.0, 0.0, 0.0)

# Game state
var chess_game: ChessGame
var selected_square: Vector2i = Vector2i(-1, -1)
var legal_moves: Array = []
var ai_plays_black: bool = false
var pending_promotion_move: Vector2i = Vector2i(-1, -1)

# Drag state
var is_dragging: bool = false
var drag_start_square: Vector2i = Vector2i(-1, -1)
var drag_mouse_pos: Vector2 = Vector2.ZERO

# UI nodes
var board_container: Control
var piece_labels: Array = []
var status_label: Label
var promotion_panel: Panel
var promotion_buttons: Dictionary = {}
var white_clock_label: Label
var black_clock_label: Label
var clock_timer: Timer
var ai_toggle: CheckBox
var clock_preset_dropdown: OptionButton

func _ready():
	DebugUtils.debug("ChessBoard _ready() called")

	# Enable mouse input for this Control node
	mouse_filter = Control.MOUSE_FILTER_STOP
	DebugUtils.debug("Mouse filter set to STOP")

	# Create the chess game instance
	chess_game = ChessGame.new()
	add_child(chess_game)
	DebugUtils.debug("ChessGame instance created and added as child")

	# Setup UI
	setup_board()
	setup_status_label()
	setup_promotion_panel()
	setup_clock_display()
	setup_clock_preset_dropdown()
	setup_ai_toggle()

	# Start with default preset (3 | 2)
	apply_clock_preset(1)

	setup_clock_timer()

	update_board()
	update_clock_display()
	DebugUtils.debug("Board setup complete")

func setup_board():
	board_container = Control.new()
	board_container.position = Vector2(20, 60)
	board_container.custom_minimum_size = Vector2(BOARD_SIZE, BOARD_SIZE)
	# Allow mouse events to pass through to parent
	board_container.mouse_filter = Control.MOUSE_FILTER_IGNORE
	add_child(board_container)

	# Create labels for pieces
	# Flip board so white is at bottom (row 0 at bottom visually)
	for row in range(8):
		var row_array = []
		for col in range(8):
			var label = Label.new()
			var display_row = 7 - row  # Flip vertically so white is at bottom
			label.position = Vector2(col * SQUARE_SIZE, display_row * SQUARE_SIZE)
			label.custom_minimum_size = Vector2(SQUARE_SIZE, SQUARE_SIZE)
			label.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
			label.vertical_alignment = VERTICAL_ALIGNMENT_CENTER

			# Set font size
			label.add_theme_font_size_override("font_size", 48)

			# CRITICAL: Allow mouse events to pass through labels to parent Control
			label.mouse_filter = Control.MOUSE_FILTER_IGNORE

			board_container.add_child(label)
			row_array.append(label)
		piece_labels.append(row_array)

func setup_status_label():
	status_label = Label.new()
	status_label.position = Vector2(20, 20)
	status_label.custom_minimum_size = Vector2(BOARD_SIZE, 30)
	status_label.add_theme_font_size_override("font_size", 24)
	add_child(status_label)
	update_status()

func setup_promotion_panel():
	# Create a panel for promotion selection
	promotion_panel = Panel.new()
	promotion_panel.position = Vector2(BOARD_SIZE / 2 - 150, BOARD_SIZE / 2 + 30)
	promotion_panel.custom_minimum_size = Vector2(300, 150)
	promotion_panel.visible = false
	add_child(promotion_panel)

	# Title label
	var title_label = Label.new()
	title_label.text = "Promote pawn to:"
	title_label.position = Vector2(10, 10)
	title_label.add_theme_font_size_override("font_size", 20)
	promotion_panel.add_child(title_label)

	# Create buttons for each piece type
	var piece_types = ["Queen", "Rook", "Bishop", "Knight"]
	var button_width = 130
	var button_height = 40
	var start_y = 50

	for i in range(piece_types.size()):
		var piece_type = piece_types[i]
		var button = Button.new()
		button.text = piece_type
		button.custom_minimum_size = Vector2(button_width, button_height)
		var col = i % 2
		var row = i / 2
		button.position = Vector2(10 + col * (button_width + 10), start_y + row * (button_height + 10))
		button.pressed.connect(_on_promotion_selected.bind(piece_type.to_lower()))
		promotion_panel.add_child(button)
		promotion_buttons[piece_type.to_lower()] = button
func setup_clock_display():
	# White clock (bottom right - where white pieces start after flip)
	white_clock_label = Label.new()
	white_clock_label.position = Vector2(BOARD_SIZE + 40, 60 + BOARD_SIZE - 60)
	white_clock_label.custom_minimum_size = Vector2(150, 50)
	white_clock_label.add_theme_font_size_override("font_size", 28)
	white_clock_label.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	white_clock_label.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
	add_child(white_clock_label)

	# Black clock (top right - where black pieces start after flip)
	black_clock_label = Label.new()
	black_clock_label.position = Vector2(BOARD_SIZE + 40, 60)
	black_clock_label.custom_minimum_size = Vector2(150, 50)
	black_clock_label.add_theme_font_size_override("font_size", 28)
	black_clock_label.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	black_clock_label.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
	add_child(black_clock_label)

func setup_clock_timer():
	clock_timer = Timer.new()
	clock_timer.wait_time = 1.0  # Tick every second
	clock_timer.timeout.connect(_on_clock_tick)
	add_child(clock_timer)
	if chess_game.has_clock():
		clock_timer.start()

func setup_clock_preset_dropdown():
	# Clock preset dropdown
	clock_preset_dropdown = OptionButton.new()
	clock_preset_dropdown.position = Vector2(BOARD_SIZE + 40, 260)
	clock_preset_dropdown.custom_minimum_size = Vector2(150, 30)
	clock_preset_dropdown.add_theme_font_size_override("font_size", 14)

	# Add preset options
	clock_preset_dropdown.add_item("1 min", 0)
	clock_preset_dropdown.add_item("3 | 2", 1)
	clock_preset_dropdown.add_item("5 min", 2)
	clock_preset_dropdown.add_item("10 min", 3)
	clock_preset_dropdown.add_item("15 | 10", 4)
	clock_preset_dropdown.add_item("30 min", 5)

	# Set default selection
	clock_preset_dropdown.selected = 1  # "3 | 2"

	clock_preset_dropdown.item_selected.connect(_on_clock_preset_changed)
	add_child(clock_preset_dropdown)

func setup_ai_toggle():
	# AI toggle checkbox
	ai_toggle = CheckBox.new()
	ai_toggle.position = Vector2(BOARD_SIZE + 40, 200)
	ai_toggle.custom_minimum_size = Vector2(150, 30)
	ai_toggle.text = "AI plays Black"
	ai_toggle.add_theme_font_size_override("font_size", 16)
	ai_toggle.toggled.connect(_on_ai_toggle_changed)
	add_child(ai_toggle)

func update_clock_display():
	if chess_game.has_clock():
		var white_time = chess_game.get_white_time()
		var black_time = chess_game.get_black_time()

		white_clock_label.text = format_time(white_time)
		black_clock_label.text = format_time(black_time)

		# Highlight active player's clock
		var current_turn = chess_game.get_current_turn()
		if current_turn == "white":
			white_clock_label.add_theme_color_override("font_color", Color(1.0, 1.0, 0.0))  # Yellow
			black_clock_label.add_theme_color_override("font_color", Color(1.0, 1.0, 1.0))  # White
		else:
			white_clock_label.add_theme_color_override("font_color", Color(1.0, 1.0, 1.0))  # White
			black_clock_label.add_theme_color_override("font_color", Color(1.0, 1.0, 0.0))  # Yellow
	else:
		white_clock_label.text = ""
		black_clock_label.text = ""

func format_time(seconds: int) -> String:
	if seconds < 0:
		return ""

	var mins = seconds / 60
	var secs = seconds % 60
	return "%d:%02d" % [mins, secs]

func _on_clock_tick():
	if not chess_game.is_game_over():
		var still_has_time = chess_game.tick_clock()
		update_clock_display()

		if not still_has_time:
			# Player ran out of time
			update_status()
			clock_timer.stop()
	else:
		clock_timer.stop()

func _draw():
	# Draw the chess board squares (flipped so white is at bottom)
	for row in range(8):
		for col in range(8):
			var is_light = (row + col) % 2 == 0
			var color = LIGHT_SQUARE_COLOR if is_light else DARK_SQUARE_COLOR
			var display_row = 7 - row  # Flip vertically
			var rect = Rect2(
				Vector2(20 + col * SQUARE_SIZE, 60 + display_row * SQUARE_SIZE),
				Vector2(SQUARE_SIZE, SQUARE_SIZE)
			)
			draw_rect(rect, color)

	# Highlight selected square
	if selected_square.x >= 0 and selected_square.y >= 0:
		var display_row = 7 - selected_square.x  # Flip vertically
		var rect = Rect2(
			Vector2(20 + selected_square.y * SQUARE_SIZE, 60 + display_row * SQUARE_SIZE),
			Vector2(SQUARE_SIZE, SQUARE_SIZE)
		)
		draw_rect(rect, SELECTED_COLOR)

	# Highlight legal move squares
	for i in range(0, legal_moves.size(), 2):
		var row = legal_moves[i]
		var col = legal_moves[i + 1]
		var display_row = 7 - row  # Flip vertically
		var rect = Rect2(
			Vector2(20 + col * SQUARE_SIZE, 60 + display_row * SQUARE_SIZE),
			Vector2(SQUARE_SIZE, SQUARE_SIZE)
		)
		draw_rect(rect, LEGAL_MOVE_COLOR)

	# Draw dragged piece following the cursor
	if is_dragging and drag_start_square.x >= 0:
		var piece = chess_game.get_piece_at(drag_start_square.x, drag_start_square.y)
		var piece_color = chess_game.get_piece_color_at(drag_start_square.x, drag_start_square.y)

		if piece != "":
			# Create a font for rendering the dragged piece
			var font = get_theme_default_font()
			var font_size = 48

			# Calculate position centered on cursor
			var text_size = font.get_string_size(piece, HORIZONTAL_ALIGNMENT_CENTER, -1, font_size)
			var draw_pos = drag_mouse_pos - text_size / 2

			# Draw piece with slight transparency to show it's being dragged
			var color = PEARL_WHITE_COLOR if piece_color == "white" else BLACK_COLOR
			color.a = 0.8  # Slight transparency
			draw_string(font, draw_pos, piece, HORIZONTAL_ALIGNMENT_CENTER, -1, font_size, color)

func update_board():
	# Update all piece labels
	for row in range(8):
		for col in range(8):
			var piece = chess_game.get_piece_at(row, col)
			var piece_color = chess_game.get_piece_color_at(row, col)

			# Hide piece if it's being dragged
			if is_dragging and drag_start_square.x == row and drag_start_square.y == col:
				piece_labels[row][col].text = ""
			else:
				piece_labels[row][col].text = piece

			# Apply color based on piece color
			if piece_color == "white":
				piece_labels[row][col].add_theme_color_override("font_color", PEARL_WHITE_COLOR)
			elif piece_color == "black":
				piece_labels[row][col].add_theme_color_override("font_color", BLACK_COLOR)

	queue_redraw()

func update_status():
	var status = chess_game.get_game_status()
	var turn = chess_game.get_current_turn()

	match status:
		"checkmate_white":
			status_label.text = "Checkmate! White wins!"
		"checkmate_black":
			status_label.text = "Checkmate! Black wins!"
		"timeloss_white":
			status_label.text = "Time out! Black wins!"
		"timeloss_black":
			status_label.text = "Time out! White wins!"
		"stalemate":
			status_label.text = "Stalemate! Draw."
		"draw":
			status_label.text = "Draw by insufficient material."
		"check":
			status_label.text = "Check! %s to move" % turn.capitalize()
		_:
			status_label.text = "%s to move" % turn.capitalize()

func _gui_input(event):
	# Check if game is over
	if chess_game.is_game_over():
		return

	# Handle mouse button press (start drag or click)
	if event is InputEventMouseButton and event.button_index == MOUSE_BUTTON_LEFT:
		var local_pos = event.position - Vector2(20, 60)

		# Check if within board bounds
		if local_pos.x < 0 or local_pos.y < 0 or local_pos.x >= BOARD_SIZE or local_pos.y >= BOARD_SIZE:
			return

		var col = int(local_pos.x / SQUARE_SIZE)
		var row = int(local_pos.y / SQUARE_SIZE)

		if event.pressed:
			# Mouse button pressed - start potential drag
			DebugUtils.debug_var("Mouse button pressed at", Vector2i(row, col))
			var piece = chess_game.get_piece_at(row, col)
			var piece_color = chess_game.get_piece_color_at(row, col)
			var current_turn = chess_game.get_current_turn()

			# Only start drag if there's a piece of the current player's color
			if piece != "" and piece_color == current_turn:
				is_dragging = true
				drag_start_square = Vector2i(row, col)
				drag_mouse_pos = event.position

				# Select the piece
				chess_game.select_piece(row, col)
				selected_square = Vector2i(row, col)
				legal_moves = chess_game.get_legal_moves_for_selected()
				queue_redraw()
		else:
			# Mouse button released - complete drag or click
			if is_dragging:
				DebugUtils.debug("Mouse button released, completing drag")
				# Try to move the piece to the drop location
				handle_piece_drop(row, col)
				is_dragging = false
				drag_start_square = Vector2i(-1, -1)
				queue_redraw()
			else:
				# Regular click (not a drag)
				handle_square_click(row, col)

	# Handle mouse motion (for dragging)
	elif event is InputEventMouseMotion and is_dragging:
		drag_mouse_pos = event.position
		queue_redraw()

func handle_piece_drop(row: int, col: int):
	DebugUtils.debug_vars({"handle_piece_drop row": row, "col": col})

	# Try to move the piece from drag_start_square to (row, col)
	if chess_game.try_move_selected(row, col):
		DebugUtils.debug("Drop successful - piece moved!")
		selected_square = Vector2i(-1, -1)
		legal_moves.clear()
		update_board()
		update_status()
		update_clock_display()
		check_ai_turn()
	else:
		DebugUtils.debug("Drop failed - invalid move")
		# Deselect the piece
		chess_game.deselect_piece()
		selected_square = Vector2i(-1, -1)
		legal_moves.clear()

func handle_square_click(row: int, col: int):
	DebugUtils.debug_vars({"handle_square_click row": row, "col": col})
	DebugUtils.debug_var("Current selected_square", selected_square)

	# If we have a piece selected, try to move it
	if selected_square.x >= 0:
		DebugUtils.debug("Piece already selected, trying to move to (%d, %d)" % [row, col])

		# Check if this is a promotion move
		if chess_game.is_promotion_move(row, col):
			# Show promotion dialog
			DebugUtils.debug("This is a promotion move, showing dialog")
			pending_promotion_move = Vector2i(row, col)
			promotion_panel.visible = true
			return

		if chess_game.try_move_selected(row, col):
			# Move was successful
			DebugUtils.debug("Move successful!")
			selected_square = Vector2i(-1, -1)
			legal_moves.clear()
			update_board()
			update_status()
			update_clock_display()
			check_ai_turn()
		else:
			DebugUtils.debug("Move failed, trying to select new piece")
			# Try to select the clicked square instead
			chess_game.deselect_piece()
			if chess_game.select_piece(row, col):
				DebugUtils.debug("New piece selected at (%d, %d)" % [row, col])
				selected_square = Vector2i(row, col)
				legal_moves = chess_game.get_legal_moves_for_selected()
				DebugUtils.debug_var("Legal moves", legal_moves)
				queue_redraw()
			else:
				DebugUtils.debug("No piece to select at (%d, %d)" % [row, col])
				selected_square = Vector2i(-1, -1)
				legal_moves.clear()
				queue_redraw()
	else:
		DebugUtils.debug("No piece selected, trying to select piece at (%d, %d)" % [row, col])
		# Try to select a piece
		if chess_game.select_piece(row, col):
			DebugUtils.debug("Piece selected successfully!")
			selected_square = Vector2i(row, col)
			legal_moves = chess_game.get_legal_moves_for_selected()
			DebugUtils.debug_var("Legal moves for selected piece", legal_moves)
			queue_redraw()
		else:
			DebugUtils.debug("Failed to select piece (empty square or wrong color)")

func _on_promotion_selected(piece_type: String):
	DebugUtils.debug_var("Promotion piece selected", piece_type)

	# Hide the promotion panel
	promotion_panel.visible = false

	# Execute the promotion move
	if pending_promotion_move.x >= 0:
		if chess_game.try_move_selected_with_promotion(pending_promotion_move.x, pending_promotion_move.y, piece_type):
			DebugUtils.debug("Promotion move successful!")
			selected_square = Vector2i(-1, -1)
			legal_moves.clear()
			pending_promotion_move = Vector2i(-1, -1)
			update_board()
			update_status()
		else:
			DebugUtils.debug("Promotion move failed!")
			pending_promotion_move = Vector2i(-1, -1)

func _on_reset_button_pressed():
	chess_game.reset_game()
	selected_square = Vector2i(-1, -1)
	legal_moves.clear()
	update_board()
	update_status()
	check_ai_turn()

func _on_ai_toggle_changed(is_checked: bool):
	ai_plays_black = is_checked
	check_ai_turn()

func _on_clock_preset_changed(index: int):
	apply_clock_preset(index)

func apply_clock_preset(preset_index: int):
	var initial_time: int
	var increment: int

	match preset_index:
		0:  # "1 min"
			initial_time = 60
			increment = 0
		1:  # "3 | 2"
			initial_time = 180
			increment = 2
		2:  # "5 min"
			initial_time = 300
			increment = 0
		3:  # "10 min"
			initial_time = 600
			increment = 0
		4:  # "15 | 10"
			initial_time = 900
			increment = 10
		5:  # "30 min"
			initial_time = 1800
			increment = 0
		_:  # Default to "3 | 2"
			initial_time = 180
			increment = 2

	chess_game.reset_game_with_clock(initial_time, increment)
	selected_square = Vector2i(-1, -1)
	legal_moves.clear()
	update_board()
	update_status()
	update_clock_display()

	# Restart clock timer
	if clock_timer:
		clock_timer.start()

	check_ai_turn()

func check_ai_turn():
	# If it's the AI's turn and game is not over, make an AI move
	if ai_plays_black and chess_game.get_current_turn() == "black" and not chess_game.is_game_over():
		make_ai_move()

func make_ai_move():
	# Small delay to make the AI move visible
	await get_tree().create_timer(0.3).timeout

	if chess_game.make_ai_move():
		update_board()
		update_status()
		update_clock_display()
