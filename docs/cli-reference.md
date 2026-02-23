# CLI Reference

## Core commands

```bash
tuitbot init
tuitbot auth
tuitbot test
tuitbot run
tuitbot tick
tuitbot stats
tuitbot settings --show
tuitbot health
```

## Tick mode options

```bash
tuitbot tick --dry-run
tuitbot tick --loops discovery,content,analytics
tuitbot tick --ignore-schedule
tuitbot tick --output json
```

## Update

```bash
tuitbot update              # check for new release + upgrade config
tuitbot update --check      # check only, don't install
tuitbot update --config-only        # skip binary, upgrade config only
tuitbot update --non-interactive    # skip all prompts
```

## Approval queue

```bash
tuitbot approve --list
tuitbot approve --approve <id>
tuitbot approve --reject <id>
tuitbot approve --approve-all
```

## Output modes

Most read-only commands support:

```bash
--output json
```

Use this for automation, monitoring, and alerting.
