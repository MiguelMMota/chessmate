extends Node

# Debug configuration
const DEBUG_ENABLED = true
const DEBUG_PREFIX = "[DEBUG]"

# Log a debug message to the console
static func debug(message: String) -> void:
	if DEBUG_ENABLED:
		print(DEBUG_PREFIX, " ", message)

# Log a debug message with a variable
static func debug_var(label: String, value) -> void:
	if DEBUG_ENABLED:
		print(DEBUG_PREFIX, " ", label, ": ", value)

# Log multiple variables in one call
static func debug_vars(values: Dictionary) -> void:
	if DEBUG_ENABLED:
		var parts = [DEBUG_PREFIX]
		for key in values:
			parts.append(str(key) + ": " + str(values[key]))
		print(" ".join(parts))
