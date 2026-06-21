import { SERVER_URL } from "../shared/server";

export default function App() {
  const handleClickSettingsButton = () => {
    chrome.runtime.openOptionsPage();
  };

  const handleClickLoginButton = () => {
    const redirectUrl = chrome.identity.getRedirectURL();
    chrome.identity.launchWebAuthFlow(
      {
        url: `${SERVER_URL}/v1/auth/login?redirect_uri=${encodeURIComponent(redirectUrl)}`,
        interactive: true,
      },
      (responseUrl) => {
        if (chrome.runtime.lastError || !responseUrl) {
          console.error(chrome.runtime.lastError);
          return;
        }
      },
    );
  };

  return (
    <>
      <button onClick={handleClickLoginButton}>Log In</button>
      <button onClick={handleClickSettingsButton}>Settings</button>
    </>
  );
}
