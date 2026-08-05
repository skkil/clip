export default function App() {
  const handleClickSettingsButton = () => {
    chrome.runtime.openOptionsPage();
  };

  return (
    <>
      <button onClick={handleClickSettingsButton}>Settings</button>
    </>
  );
}
