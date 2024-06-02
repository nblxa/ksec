&& ret=0
  # create a new var for 'words' trimmed from index 1 up to the word previous to the current word
  local params=("${words[@]:1:$(($CURRENT-2))}")
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
      _values 'contexts' $(ksec --completion-helper context x $params)
      ;;
    namespaces)
      _values  'namespaces' $(ksec --completion-helper namespace x $params)
      ;;
    secrets)
      _values 'secrets' $(ksec --completion-helper secret x $params)
      ;;
    keys)
      _values 'keys' $(ksec --completion-helper key $params)
      ;;
    *)
      ;;
  esac
