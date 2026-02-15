from garminconnect import Garmin
from .api_safe import safe_api_call
import logging

logger = logging.getLogger(__name__)


class GarminClient:
    def __init__(self, config):
        self.config = config
        self.api = None

    def load_session(self):
        try:
            api = Garmin()
            api.garth.load(self.config.tokenstore)
            self.api = api
            return True
        except Exception:
            return False

    def login(self, mfa_callback=None):
        # 1. Try tokenstore first
        if self.load_session():
            return

        # 2. Credential login
        api = Garmin(
            self.config.email,
            self.config.password,
            return_on_mfa=True,
        )

        result1, result2 = api.login()

        if result1 == "needs_mfa":
            if not callable(mfa_callback):
                raise Exception("MFA required")
            api.resume_login(result2, mfa_callback())

        # 3. Persist tokens
        api.garth.dump(self.config.tokenstore)
        self.api = api

    def is_connected(self):
        return self.api is not None

    def _call(self, methods, *args, **kwargs):
        logger.debug("Calling Garmin endpoint: %s", methods.__name__)

        success, data, err = safe_api_call(methods, *args, **kwargs)

        if not success:
            logger.error("Garmin API call fialed: %s", err)
            raise Exception(err)

        return data

    def get_activities(self, start, end):
        if not self.is_connected():
            raise RuntimeError("Client is not connected")

        data = self._call(self.api.get_activities_by_date, start, end)
        print(data)

        if not data:
            return str("you haven't done anything dude get moving!")

        return data

    def get_activity(self, activity_id):
        if not self.is_connected():
            raise RuntimeError("Client is not connected")

        return self._call(self.api.get_activity_details, activity_id)
