import subprocess
import shutil
import functools


def notify(title, text):
    try:
        if shutil.which("notify-send"):
            subprocess.run(["notify-send", title, text])
        elif shutil.which("osascript"):
            subprocess.run([
                "osascript", "-e",
                f'display notification "{text}" with title "{title}"'
            ])
        else:
            print("📢 Notification not supported on this OS.")
    except Exception as e:
        print(f"⚠️ Notification failed: {e}")


def notify_on_finish(title="Task Complete", text="The task has finished."):
    """Decorator to notify the user when the task is complete."""

    def decorator(fn):

        @functools.wraps(fn)
        def wrapper(c, *args, **kwargs):
            if not hasattr(c, "main_task"):
                c.main_task = True

            try:
                result = fn(c, *args, **kwargs)

                if getattr(c, "main_task", False):
                    notify(title, text)
            except Exception:
                notify("Error", "An error occurred.")
                raise
            return result

        return wrapper
    return decorator
