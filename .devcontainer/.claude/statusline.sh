#!/bin/bash
input=$(cat)

reset="\033[0m"
dim="\033[38;5;245m"
blue="\033[34m"
green="\033[32m"

# Parse JSON fields
model=$(echo "$input" | jq -r '.model.display_name')
cost=$(echo "$input" | jq -r '.cost.total_cost_usd // "0"')
formatted_cost=$(printf "%.2f" "$cost")

# Context window
context_size=$(echo "$input" | jq -r '.context_window.context_window_size // 200000')
usage=$(echo "$input" | jq '.context_window.current_usage // null')

if [ "$usage" != "null" ]; then
    current_tokens=$(echo "$usage" | jq '.input_tokens + .cache_creation_input_tokens + .cache_read_input_tokens')
    pct=$((current_tokens * 100 / context_size))
else
    current_tokens=0
    pct=0
fi

if [ "$current_tokens" -lt 1000 ]; then
    token_display="${current_tokens}"
elif [ "$current_tokens" -lt 10000 ]; then
    token_display=$(awk "BEGIN {printf \"%.1fk\", $current_tokens/1000}")
else
    token_display="$((current_tokens / 1000))k"
fi

if [ "$pct" -lt 70 ]; then
    color="38;5;29"
elif [ "$pct" -lt 85 ]; then
    color="38;5;220"
else
    color="38;5;208"
fi

token_colored=$(printf "\033[%sm%s\033[0m" "$color" "$token_display")

# Git branch + repo name
branch=$(git branch --show-current 2>/dev/null)
repo=$(basename "$(git rev-parse --show-toplevel 2>/dev/null)" 2>/dev/null)
if [ -n "$branch" ]; then
    branch_part=$(printf "${blue}%s${reset} ${dim}🌿${reset} ${green}%s${reset}" "$repo" "$branch")
    printf "📦 ${reset}%s ${dim}|${reset} %s ${dim}|${reset} %s ${dim}|${reset} \$%s" \
        "$model" "$branch_part" "$token_colored" "$formatted_cost"
else
    printf "📦 ${reset}%s ${dim}|${reset} %s ${dim}|${reset} \$%s" \
        "$model" "$token_colored" "$formatted_cost"
fi
