from garminconnect import Garmin
from .api_safe import safe_api_call
import logging

logger = logging.getLogger(__name__)


class GarminClient:
    def __init__(self, config):
        self.config = config
        self.api = None

    def login(self, mfa_callback=None):

        success, api, err = safe_api_call(
            Garmin, self.config.email, self.config.password
        )

        if success:
            self.api = api
            return True, None, None

        # MFA required
        if "mfa" in str(err).lower():
            if not callable(mfa_callback):
                return False, None, "MFA required but no callback provided"

            code = mfa_callback()

            success, api, err = safe_api_call(
                Garmin, self.config.email, self.config.password, code
            )

            if not success:
                return False, None, err

            self.api = api
            return True, None, None

        return False, None, err

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

        return self._call(self.api.get_activities_by_date, start, end)

    def get_activity(self, activity_id):
        if not self.is_connected():
            raise RuntimeError("Client is not connected")

        return self._call(self.api.get_activity_details, activity_id)
