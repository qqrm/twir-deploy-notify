# Restart Command

The repository uses an AI agent to generate merge requests. When the `Restart` command is issued, the agent copies the description of the current task and creates a new one that starts from the latest commit. The agent then prints:

```
Task restarted. Launch it on a clean commit? (Yes/No)
```

Confirming will run the workflow manually on the fresh commit, avoiding merge conflicts.
