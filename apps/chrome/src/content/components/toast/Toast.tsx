import styles from './Toast.module.css';

export type ToastStatus = 'success' | 'error' | 'info';

export interface ToastMessage {
  status: ToastStatus;
  text: string;
}

interface ToastProps {
  message: ToastMessage | null;
  visible: boolean;
}

function Icon({ status }: { status: ToastStatus }) {
  if (status === 'success') {
    return (
      <svg viewBox="0 0 16 16" width="13" height="13" aria-hidden="true">
        <path
          d="M3.5 8.5l3 3 6-7"
          fill="none"
          stroke="currentColor"
          strokeWidth="2"
          strokeLinecap="round"
          strokeLinejoin="round"
        />
      </svg>
    );
  }

  if (status === 'error') {
    return (
      <svg viewBox="0 0 16 16" width="13" height="13" aria-hidden="true">
        <path
          d="M4 4l8 8M12 4l-8 8"
          fill="none"
          stroke="currentColor"
          strokeWidth="2"
          strokeLinecap="round"
        />
      </svg>
    );
  }

  return (
    <svg viewBox="0 0 16 16" width="13" height="13" aria-hidden="true">
      <circle
        cx="8"
        cy="8"
        r="6"
        fill="none"
        stroke="currentColor"
        strokeWidth="2"
      />
    </svg>
  );
}

export function Toast({ message, visible }: ToastProps) {
  if (!message) {
    return null;
  }

  return (
    <div
      className={`${styles.toast} ${visible ? styles.visible : ''}`}
      role="status"
      aria-live="polite"
    >
      <span className={`${styles.icon} ${styles[message.status]}`}>
        <Icon status={message.status} />
      </span>
      <span className={styles.text}>{message.text}</span>
    </div>
  );
}
