#!/bin/zsh
fn() {#trim
  local state #trim
  # create a new var for 'words' trimmed from index 1 up to the word previous to the current word
  local params=("${words[@]:1:$((CURRENT-2))}")
  # if the last item in params is an option that accepts a value (-c, -n etc.), remove it
  if [[ "${params[-1]}" == -* ]]; then
    case "${params[-1]}" in
      --kubeconfig|-c|--context|-n|--namespace|--completion)
        params=("${params[@]:0:$(($#params-1))}")
        ;;
      *)
        ;;
    esac
  fi
  case "$state" in
    contexts)
      #shellcheck disable=SC2128 disable=SC2086 disable=SC2207
      typeset -a contexts=($(ksec --completion-helper=contexts x $params))
      if [ ${#contexts[@]} -ne 0 ]; then
        _values 'contexts' "${contexts[@]}"
      else
        _message 'no contexts found'
      fi
      ;;
    namespaces)
      #shellcheck disable=SC2128 disable=SC2086 disable=SC2207
      typeset -a namespaces=($(ksec --completion-helper=namespaces x $params))
      if [ ${#namespaces[@]} -ne 0 ]; then
        _values  'namespaces' "${namespaces[@]}"
      else
        _message 'no namespaces found'
      fi
      ;;
    secrets)
      #shellcheck disable=SC2128 disable=SC2086 disable=SC2207
      typeset -a secrets=($(ksec --completion-helper=secrets x $params))
      if [ ${#secrets[@]} -ne 0 ]; then
        _values 'secrets' "${secrets[@]}"
      else
        _message 'no secrets found'
      fi
      ;;
    keys)
      #shellcheck disable=SC2128 disable=SC2086 disable=SC2207
      typeset -a keys=($(ksec --completion-helper=keys $params))
      if [ ${#keys[@]} -ne 0 ]; then
        _values 'keys' "${keys[@]}"
      else
        _message 'no keys found'
      fi
      ;;
    *)
      ;;
  esac
}#trim
