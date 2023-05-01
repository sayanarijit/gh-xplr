local xplr = xplr
local state = {
  search = "",
  repos = {},
}

xplr.config.modes.builtin.default.key_bindings.on_key["enter"] = {
  help = "browse",
  messages = {
    {
      BashExecSilently0 = [===[
        while read -d '' -r path; do

          dirname=$(dirname "$path")
          basename=$(basename "$path")

          cd "${dirname:?}"

          if [ -e "$basename" ]; then
              gh browse "$basename"
              url=$(gh browse -n "$basename")
              "$XPLR" -m "LogSuccess: %q" "$url"
          else
              gh browse .
              url=$(gh browse -n .)
              "$XPLR" -m "LogSuccess: %q" "$url"
          fi
        done < "$XPLR_PIPE_RESULT_OUT"
      ]===],
    },
    "ClearSelection",
  },
}

xplr.config.modes.builtin.go_to.key_bindings.on_key["r"] = {
  help = "repository",
  messages = {
    "PopMode",
    { CallLuaSilently = "custom.go_to_repository.init" },
  },
}

xplr.config.modes.custom.go_to_repository = {
  name = "go to repository",
  layout = {
    Horizontal = {
      config = {
        constraints = {
          { Percentage = 50 },
          { Percentage = 50 },
        },
      },
      splits = {
        {
          Vertical = {
            config = {
              constraints = {
                { Min = 1 },
                { Length = 3 },
              },
            },
            splits = {
              { Dynamic = "custom.go_to_repository.render" },
              "InputAndLogs",
            },
          },
        },
        "HelpMenu",
      },
    },
  },
  key_bindings = {
    on_key = {
      ["tab"] = {
        help = "search",
        messages = {
          { CallLuaSilently = "custom.go_to_repository.search" },
        },
      },
      ["up"] = {
        help = "up",
        messages = {
          { CallLuaSilently = "custom.go_to_repository.up" },
        },
      },
      ["down"] = {
        help = "down",
        messages = {
          { CallLuaSilently = "custom.go_to_repository.down" },
        },
      },
      ["enter"] = {
        help = "submit",
        messages = {
          {
            BashExec0 = [===[
              dir="$(mktemp -d)"
              if gh repo clone "${XPLR_INPUT_BUFFER:?}" "$dir" -- --depth 1; then
                "$XPLR" -m 'UnsetVroot'
                "$XPLR" -m 'ChangeDirectory: %q' "$dir"
                "$XPLR" -m 'SetVroot: %q' "$dir"
              else
                read -p "[press ENTER to continue]"
              fi
            ]===],
          },
          "PopMode",
        },
      },
    },
    default = {
      messages = {
        "UpdateInputBufferFromKey",
      },
    },
  },
}

xplr.config.modes.custom.go_to_repository.key_bindings.on_key["j"] =
    xplr.config.modes.builtin.default.key_bindings.on_key["down"]
xplr.config.modes.custom.go_to_repository.key_bindings.on_key["k"] =
    xplr.config.modes.builtin.default.key_bindings.on_key["up"]
xplr.config.modes.custom.go_to_repository.key_bindings.on_key["back-tab"] =
    xplr.config.modes.builtin.default.key_bindings.on_key["up"]

xplr.fn.custom.go_to_repository = {}

xplr.fn.custom.go_to_repository.init = function(_)
  state.search = ""
  state.repos = {}
  return {
    { SwitchModeCustom = "go_to_repository" },
    { SetInputBuffer = state.search },
  }
end

xplr.fn.custom.go_to_repository.search = function(app)
  local exists = 0

  for i, repo in ipairs(state.repos) do
    if app.input_buffer == repo then
      exists = i
      break
    end
  end

  if exists ~= 0 then
    return xplr.fn.custom.go_to_repository.down(app, exists)
  end

  state.search = app.input_buffer

  local res = xplr.util.shell_execute(
    "gh",
    { "search", "repos", "--json", "fullName", "--limit", "10", "--", state.search }
  )
  if res.returncode ~= 0 then
    return {
      { LogError = res.stderr },
    }
  end

  local repos = xplr.util.from_json(res.stdout)
  for i, repo in ipairs(repos) do
    state.repos[i] = repo.fullName
  end

  if #state.repos == 0 then
    state.repos = {}
  else
    return {
      { SetInputBuffer = state.repos[1] },
    }
  end
end

xplr.fn.custom.go_to_repository.down = function(app, i)
  local index = 1
  if i ~= nil then
    index = i
  else
    for i_, repo in ipairs(state.repos) do
      if app.input_buffer == repo then
        index = i_
        break
      end
    end
  end

  if index == #state.repos then
    return {
      { SetInputBuffer = state.repos[1] },
    }
  else
    return {
      { SetInputBuffer = state.repos[index + 1] },
    }
  end
end

xplr.fn.custom.go_to_repository.up = function(app)
  local index = 1
  for i, repo in ipairs(state.repos) do
    if app.input_buffer == repo then
      index = i
      break
    end
  end

  if index == 1 then
    return {
      { SetInputBuffer = state.repos[#state.repos] },
    }
  else
    return {
      { SetInputBuffer = state.repos[index - 1] },
    }
  end
end

xplr.fn.custom.go_to_repository.render = function(ctx)
  local title = " press <tab> to search "
  if state.search ~= "" then
    title = " search:" .. state.search .. " "
  end

  local repos = {}

  for i, repo in ipairs(state.repos) do
    if ctx.app.input_buffer == repo then
      local reversed = { add_modifiers = { "Reversed" } }
      repos[i] = xplr.util.paint(repo, reversed)
    else
      repos[i] = repo
    end
  end

  if #repos == 0 then
    repos = { " ", " no results..." }
  end

  return {
    CustomList = {
      ui = { title = { format = title } },
      body = repos,
    },
  }
end
