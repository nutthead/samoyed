**Context:**
  Samoyed is designed to simplify and streamline management of git hooks.
  Its CLI is very simple: `samoyed init [samoyed-dirname]`, where `[samoyed-dirname]` is an optional argument and its value is the directory where Samoyed will set up its files (default: `.samoyed`).
  Furthermore, Samoyed modifies the git repository's `.git/config` file to point to the hooks directory inside the specified Samoyed directory.
  Consequently, it **MUST NOT** run inside its own git repository directory, as depending on how it is executed, it may unexpectedly overwrite the hooks setup for the repository, create additional hooks directories
  
  Henceforth, integration tests for Samoyed should be executed in a temporary directory. We have created the `./tmp/` directory for this purpose, which is already included in `.gitignore`.
  We want to keep this directory in the repository, so we have added a `.gitkeep` file inside it. However, everything else inside `./tmp/` IS be ignored by git.
  This can be verified by reading the `.gitignore` file.

**Task:**
  We want to write integration tests for Samoyed. As Newton said, "If I have seen further it is by standing on the shoulders of Giants."
  So instead of reinventing the wheel, we read the integration tests for another project named `husky`, which is another popular git hooks manager written in Node/JavaScript.
  We have cloned the `husky` repository into `/home/amadeus/Code/ot/husky` and its integration tests are located in the `/home/amadeus/Code/ot/husky/test` directory. All these tests are written in POSIX shell script.
  Read the integration tests in `/home/amadeus/Code/ot/husky/test` and systematically, methodically, and carefully port them to test Samoyed.
  Just to reiterate, the integration tests for Samoyed MUST be written in such a way that they CAN ONLY modify files inside the `./tmp/` directory (with the exception of the `.gitkeep` file).
  Whenever you need to refresh your memory about any topic, or fix your failures and mistakes, use the web searching and web fetching capabilities of Claude Code (i.e. `WebSearch` and `WebFetch`).
  Before you start, created a comprehensive TODO list using the `TodoWrite` tool, and then follow the TODO list diligently. However, when you think the TODO list must be modified, feel free to modify it, then continue your work towards accomplishing the task.
  Adopt these mindsets throughout the task: methodical, systematic, careful, thorough, diligent, precise, accurate, detail-oriented, and perfectionist.

**Approach**
  1. Read Samoyed's source code and unit tests in `src/main.rs` to understand how it works.
  2. Read the integration tests in `/home/amadeus/Code/ot/husky/test` and understand what each test does.
  3. For each test, write a corresponding test for Samoyed that performs the same functionality but only modifies files inside the `./tmp/` directory. If the test does not make sense for Samoyed (for example, if it relies on Node.js-specific features), skip it and move on to the next test.
  4. Ensure that the tests are written in POSIX shell script and follow best practices for writing shell scripts.
  5. Ensure the shell scripts are very well-commented to explain what each part of the test does.
  6. After porting each test, lint it using `shellcheck` to ensure it follows best practices and is free of common mistakes. However, do not blindly follow `shellcheck` suggestions if they contradict the intended functionality of the test and feel free to disable specific `shellcheck` rules using comments if necessary.
  7. After finishing each test, run it to ensure it works as expected and if it found any bugs in Samoyed, fix them in `src/main.rs` and/or `assets/samoyed`.
  8. Finally, run the tests to ensure they work as expected.

_Start!_