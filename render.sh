#!/bin/bash

# Function to render a JSON element
render_element() {
	local element="$1"

	case $(jq -r '.type' <<<"$element") in
	"Root")
		# Handle root element (optional)
		;;
	"Heading")
		echo -e "\033[32m## $(jq -r '.children[0].value' <<<"$element")\033[0m"
		;;
	"Paragraph")
		echo -e "\033[36m$(jq -r '.children[0].value' <<<"$element")\033[0m"
		;;
	"Code")
		echo -e "\033[33m$(\"$(jq -r '.value' <<<"$element")\")\033[0m"
		;;
	"Table")
		# Handle table elements (more complex logic required)
		;;
	# Add more cases for other element types as needed
	*)
		echo "Unsupported element type: $(jq -r '.type' <<<"$element")"
		;;
	esac
}

# Function to recursively render a JSON structure
render_json() {
	local json_data="$1"

	# Use jq with the '-e' option to exit on errors
	if ! jq -r '.children[]' <<<"$json_data"; then
		echo "Error: Invalid JSON input"
		return 1 # Indicate error
	fi

	while read -r element; do
		render_element "$element"
	done < <(jq -r '.children[]' <<<"$json_data")
}

# Read JSON input from a file or standard input
if [[ "$1" ]]; then
	json_data=$(cat "$1")
else
	read -r json_data
fi

# Render the JSON structure
render_json "$json_data"
