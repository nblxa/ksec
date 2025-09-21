#!/bin/bash
# Smart completion logic for ksec bash completion

_ksec_smart_completion() {
    local cur="${COMP_WORDS[COMP_CWORD]}"
    local prev="${COMP_WORDS[COMP_CWORD-1]}"

    # Build parameters array excluding the current word
    local params=()
    local i=1
    while [[ $i -lt $((COMP_CWORD)) ]]; do
        params+=("${COMP_WORDS[$i]}")
        ((i++))
    done

    # Remove the last parameter if it's an option that takes a value
    if [[ ${#params[@]} -gt 0 ]]; then
        case "${params[-1]}" in
            --kubeconfig|-c|--context|-n|--namespace|--completion)
                # Use array slice instead of unset for bash 3 compatibility
                params=("${params[@]:0:$((${#params[@]}-1))}")
                ;;
        esac
    fi

    case "${prev}" in
        -c|--context)
            # Complete contexts - bash 3 compatible approach
            local contexts_output contexts
            contexts_output=$(ksec --completion-helper=contexts dummy "${params[@]}" 2>/dev/null)
            if [[ -n "$contexts_output" ]]; then
                # Convert newline-separated output to array (bash 3 compatible)
                local IFS=$'\n'
                #shellcheck disable=SC2206
                contexts=($contexts_output)
                # shellcheck disable=SC2207
                COMPREPLY=($(compgen -W "${contexts[*]}" -- "${cur}"))
            fi
            return 0
            ;;
        -n|--namespace)
            # Complete namespaces - bash 3 compatible approach
            local namespaces_output namespaces
            namespaces_output=$(ksec --completion-helper=namespaces dummy "${params[@]}" 2>/dev/null)
            if [[ -n "$namespaces_output" ]]; then
                # Convert newline-separated output to array (bash 3 compatible)
                local IFS=$'\n'
                #shellcheck disable=SC2206
                namespaces=($namespaces_output)
                #shellcheck disable=SC2207
                COMPREPLY=($(compgen -W "${namespaces[*]}" -- "${cur}"))
            fi
            return 0
            ;;
    esac

    # Count positional arguments (excluding options)
    local pos_args=0
    for arg in "${params[@]}"; do
        if [[ ! "$arg" =~ ^- ]]; then
            ((pos_args++))
        fi
    done

    # Complete based on positional argument position
    case $pos_args in
        0)
            # Complete secret names - bash 3 compatible approach
            local secrets_output secrets
            secrets_output=$(ksec --completion-helper=secrets dummy "${params[@]}" 2>/dev/null)
            if [[ -n "$secrets_output" ]]; then
                # Convert newline-separated output to array (bash 3 compatible)
                local IFS=$'\n'
                #shellcheck disable=SC2206
                secrets=($secrets_output)
                #shellcheck disable=SC2207
                COMPREPLY=($(compgen -W "${secrets[*]}" -- "${cur}"))
            fi
            ;;
        1)
            # Complete secret keys - bash 3 compatible approach
            local keys_output keys
            keys_output=$(ksec --completion-helper=keys "${params[@]}" 2>/dev/null)
            if [[ -n "$keys_output" ]]; then
                # Convert newline-separated output to array (bash 3 compatible)
                local IFS=$'\n'
                #shellcheck disable=SC2206
                keys=($keys_output)
                #shellcheck disable=SC2207
                COMPREPLY=($(compgen -W "${keys[*]}" -- "${cur}"))
            fi
            ;;
    esac
}
