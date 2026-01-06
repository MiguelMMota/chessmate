extends Control

# Board visual properties
const BOARD_SIZE = 640
const SQUARE_SIZE = BOARD_SIZE / 8
const LIGHT_SQUARE_COLOR = Color(0.93, 0.85, 0.71)
const DARK_SQUARE_COLOR = Color(0.72, 0.53, 0.38)
const SELECTED_COLOR = Color(0.8, 0.8, 0.3, 0.5)
const LEGAL_MOVE_COLOR = Color(0.3, 0.8, 0.3, 0.5)

# Game state
var chess_game: ChessGame
var selected_square: Vector2i = Vector2i(-1, -1)
var legal_moves: Array = []

# UI nodes
var board_container: Control
var piece_labels: Array = []
var status_label: Label

func _ready():
	# Enable mouse input for this Control node
	mouse_filter = Control.MOUSE_FILTER_STOP

	# Create the chess game instance
	chess_game = ChessGame.new()
	add_child(chess_game)

	# Setup UI
	setup_board()
	setup_status_label()
	update_board()

func setup_board():
	board_container = Control.new()
	board_container.position = Vector2(20, 60)
	board_container.custom_minimum_size = Vector2(BOARD_SIZE, BOARD_SIZE)
	add_child(board_container)

	# Create labels for pieces
	for row in range(8):
		var row_array = []
		for col in range(8):
			var label = Label.new()
			label.position = Vector2(col * SQUARE_SIZE, row * SQUARE_SIZE)
			label.custom_minimum_size = Vector2(SQUARE_SIZE, SQUARE_SIZE)
			label.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
			label.vertical_alignment = VERTICAL_ALIGNMENT_CENTER

			# Set font size
			label.add_theme_font_size_override("font_size", 48)

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

func _draw():
	# Draw the chess board squares
	for row in range(8):
		for col in range(8):
			var is_light = (row + col) % 2 == 0
			var color = LIGHT_SQUARE_COLOR if is_light else DARK_SQUARE_COLOR
			var rect = Rect2(
				Vector2(20 + col * SQUARE_SIZE, 60 + row * SQUARE_SIZE),
				Vector2(SQUARE_SIZE, SQUARE_SIZE)
			)
			draw_rect(rect, color)

	# Highlight selected square
	if selected_square.x >= 0 and selected_square.y >= 0:
		var rect = Rect2(
			Vector2(20 + selected_square.y * SQUARE_SIZE, 60 + selected_square.x * SQUARE_SIZE),
			Vector2(SQUARE_SIZE, SQUARE_SIZE)
		)
		draw_rect(rect, SELECTED_COLOR)

	# Highlight legal move squares
	for i in range(0, legal_moves.size(), 2):
		var row = legal_moves[i]
		var col = legal_moves[i + 1]
		var rect = Rect2(
			Vector2(20 + col * SQUARE_SIZE, 60 + row * SQUARE_SIZE),
			Vector2(SQUARE_SIZE, SQUARE_SIZE)
		)
		draw_rect(rect, LEGAL_MOVE_COLOR)

func update_board():
	# Update all piece labels
	for row in range(8):
		for col in range(8):
			var piece = chess_game.get_piece_at(row, col)
			piece_labels[row][col].text = piece

	queue_redraw()

func update_status():
	var status = chess_game.get_game_status()
	var turn = chess_game.get_current_turn()

	match status:
		"checkmate_white":
			status_label.text = "Checkmate! White wins!"
		"checkmate_black":
			status_label.text = "Checkmate! Black wins!"
		"stalemate":
			status_label.text = "Stalemate! Draw."
		"draw":
			status_label.text = "Draw by insufficient material."
		"check":
			status_label.text = "Check! %s to move" % turn.capitalize()
		_:
			status_label.text = "%s to move" % turn.capitalize()

func _gui_input(event):
	if event is InputEventMouseButton and event.pressed and event.button_index == MOUSE_BUTTON_LEFT:
		# Check if game is over
		if chess_game.is_game_over():
			return

		# Convert mouse position to board coordinates
		var local_pos = event.position - Vector2(20, 60)
		if local_pos.x < 0 or local_pos.y < 0 or local_pos.x >= BOARD_SIZE or local_pos.y >= BOARD_SIZE:
			return

		var col = int(local_pos.x / SQUARE_SIZE)
		var row = int(local_pos.y / SQUARE_SIZE)

		handle_square_click(row, col)

func handle_square_click(row: int, col: int):
	# If we have a piece selected, try to move it
	if selected_square.x >= 0:
		if chess_game.try_move_selected(row, col):
			# Move was successful
			selected_square = Vector2i(-1, -1)
			legal_moves.clear()
			update_board()
			update_status()
		else:
			# Try to select the clicked square instead
			chess_game.deselect_piece()
			if chess_game.select_piece(row, col):
				selected_square = Vector2i(row, col)
				legal_moves = chess_game.get_legal_moves_for_selected()
				queue_redraw()
			else:
				selected_square = Vector2i(-1, -1)
				legal_moves.clear()
				queue_redraw()
	else:
		# Try to select a piece
		if chess_game.select_piece(row, col):
			selected_square = Vector2i(row, col)
			legal_moves = chess_game.get_legal_moves_for_selected()
			queue_redraw()

func _on_reset_button_pressed():
	chess_game.reset_game()
	selected_square = Vector2i(-1, -1)
	legal_moves.clear()
	update_board()
	update_status()
