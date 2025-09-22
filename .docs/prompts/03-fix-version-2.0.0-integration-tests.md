POSIX shell (e.g., `/bin/sh`, `dash`, `ash`) does not support the `local` keyword for declaring local variables within functions. This can lead to variable name collisions and unintended side effects when functions modify global variables. Study the following strategies to manage variable scope effectively in POSIX shell scripts, then choose the **right strategy** on a case by case basis, and then update the shell scripts written under `tests/integration` accordingly. Ensure the updated scripts are well-commented AND don't have useless comments. Verify the updated scripts with `shellcheck` and fix any issues it raises, unless they are false positives or contradict the intended functionality of the script. If you need to disable specific `shellcheck` rules, do so using comments in the script.

# Workarounds Used by POSIX Shell Writers

Since `local` is not available, POSIX-compliant shell scripters use several strategies to simulate local variables or avoid variable pollution:

## 1. **Prefix Variable Names with Function Name**

The most common and practical workaround is to **prefix variable names** with the function name (or an abbreviation) to avoid collisions.

```sh
myfunc() {
    myfunc_i=0
    myfunc_temp="hello"
    # ... use myfunc_i, myfunc_temp ...
    unset myfunc_i myfunc_temp # optional cleanup
}
```

✅ **Pros**: Simple, portable, readable.  
❌ **Cons**: Verbose, manual cleanup needed if desired.

## 2. **Use Subshells to Isolate Variables**

Run the function body (or part of it) in a **subshell**. Variables inside a subshell don’t affect the parent.

```sh
myfunc() (
    i=0
    temp="hello"
    # ... variables are local to this subshell ...
    # no need to unset — they vanish when subshell exits
)
```

Note: `()` instead of `{}` — this creates a subshell.

✅ **Pros**: Truly local variables, automatic cleanup.  
❌ **Cons**:
- Cannot modify global variables or return values via variables (only exit status or stdout).
- Slightly slower (fork overhead).
- Cannot `cd`, `export`, or affect parent shell state.

## 3. **Pass State via stdout / Exit Status Only**

Design functions to communicate only via **stdout** and **exit codes**, avoiding variable side effects entirely.

```sh
get_temp_dir() {
    printf '/tmp/myapp.%s' "$$"
}

dir=$(get_temp_dir)
```

This functional style avoids the need for local variables altogether in many cases.

---

## 4. **Avoid Reusing Common Variable Names**

Just don’t use generic names like `i`, `temp`, `result`, `file`, etc., without prefixes. Use descriptive, scoped names.

```sh
process_file() {
    process_file_index=0
    process_file_line=""
    # ...
}
```

---

## 🚫 What NOT to Do

- You MUST NOT assume `local` exists — it will break on `/bin/sh` (e.g., dash, ash, POSIX mode bash).
  - YOU MUST NOT use `local` in POSIX shell scripts.
- Don’t rely on `typeset` — not in POSIX (though available in ksh/bash, it’s not portable).

## Best Practices Summary

| Strategy                       | Use When...                              | Notes                         |
|--------------------------------|------------------------------------------|-------------------------------|
| Prefix variable names          | Always — safest, most portable           | Manual cleanup optional       |
| Subshells `( )`                | Function is side-effect free             | Can’t modify parent state     |
| `unset` at end                 | You need to modify globals               | Risky if function exits early |
| Functional style (stdout)      | Returning simple values                  | Clean, composable             |
| Avoid generic var names        | Works everywhere                         | Prevents collisions           |

---

## Example: Combining Strategies

```sh
calculate_sum() {
    calc_i=0
    calc_total=0
    calc_arg=""

    for calc_arg in "$@"; do
        calc_total=$((calc_total + calc_arg))
    done

    printf '%s\n' "$calc_total"

    unset calc_i calc_total calc_arg
}
```

Or with subshell (if no side effects needed):

```sh
calculate_sum() (
    total=0
    for arg in "$@"; do
        total=$((total + arg))
    done
    printf '%s\n' "$total"
    # variables auto-vanish
)
```

---

## Conclusion

POSIX shell programmers simulate local variables by:

- **Naming conventions** (prefixes),
- **Subshells** for isolation,
- **Functional patterns** to avoid side effects.

While more verbose than `local`, these techniques are robust, portable, and widely used in production POSIX shell scripts (e.g., in `/bin/sh` scripts on Linux/BSD, init scripts, installers, etc.).

Stick with prefixes + subshells as needed — it’s the POSIX way. 🐚

---

As you update the integration test scripts under `tests/integration`, explain what you are doing and the rationale behind your choices.
