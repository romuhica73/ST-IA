import { MediaDropZone } from "./features/media-selection/MediaDropZone";
import { SelectedMedia } from "./features/media-selection/SelectedMedia";
import { useMediaSelection } from "./features/media-selection/useMediaSelection";
import "./styles/App.css";

function App() {
  const { state, selectViaDialog, reset } = useMediaSelection();

  return (
    <main className="app">
      <header className="app__header">
        <h1>ST-IA</h1>
      </header>

      {state.status === "selected" ? (
        <SelectedMedia
          media={state.media}
          onChangeFile={selectViaDialog}
          onRemove={reset}
        />
      ) : (
        <MediaDropZone
          isDragging={state.status === "dragging"}
          errorMessage={state.status === "error" ? state.message : null}
          onSelectClick={selectViaDialog}
        />
      )}
    </main>
  );
}

export default App;
