import subprocess
import shutil
import functools

def notify_on_finish(fn):
    """Decorator to notify the user when the task is complete."""
    @functools.wraps(fn)
    def wrapper(c, *args, **kwargs):
        # If not already marked, assume this is the main entry point
        if not hasattr(c, "main_task"):
            c.main_task = True
        result = fn(c, *args, **kwargs)

        if getattr(c, "main_task", False):
            return
        try:
            if shutil.which("notify-send"):
                subprocess.run(["notify-send", "Benchmark Complete", "Your benchmark has finished."])
            elif shutil.which("osascript"):
                subprocess.run([
                    "osascript", "-e",
                    'display notification "Your benchmark has finished." with title "Benchmark Complete"'
                ])
            else:
                print("📢 Notification not supported on this OS.")
        except Exception as e:
            print(f"⚠️ Notification failed: {e}")

        return result
    return wrapper
