import { MediaDropZone } from "./features/media-selection/MediaDropZone";
import { SelectedMedia } from "./features/media-selection/SelectedMedia";
import { useMediaSelection } from "./features/media-selection/useMediaSelection";
import "./styles/App.css";

function App() {
  const { state, selectViaDialog } = useMediaSelection();

  return (
    <main className="app">
      {state.status === "selected" ? (
        <SelectedMedia media={state.media} onChangeFile={selectViaDialog} />
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
