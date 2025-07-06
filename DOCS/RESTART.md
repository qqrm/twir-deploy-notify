# Restart Command

The repository uses an AI agent to generate merge requests. When the `Restart` command is issued, the agent copies the description of the current task and prepares a new **task stub** based on the freshest `main` commit. The agent then prints:

```
Task restarted as a stub from the latest commit. Launch it? (Yes/No)
```

Confirming will run the workflow manually on the fresh commit, avoiding merge conflicts.
