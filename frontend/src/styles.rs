pub struct Styles;

impl Styles {
    pub const GLOBAL_STYLE: &'static str = r#"
        :root {
            color-scheme: light;
            --bg: #fff8ef;
            --bg-soft: #fff2df;
            --panel: rgba(255, 255, 255, 0.82);
            --panel-strong: #fffdf8;
            --ink: #2d2118;
            --muted: #756252;
            --border: rgba(94, 66, 38, 0.12);
            --shadow: 0 20px 60px rgba(124, 72, 18, 0.16);
            --primary: #f26a3d;
            --primary-dark: #d6562e;
            --accent: #ffb34d;
            --success: #3f9b68;
            --warning: #d8932f;
            --offline: #b25a62;
        }

        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }

        html,
        body {
            width: 100%;
            min-height: 100%;
            background:
                radial-gradient(circle at top left, rgba(255, 199, 125, 0.4), transparent 32%),
                radial-gradient(circle at right 15%, rgba(242, 106, 61, 0.18), transparent 24%),
                linear-gradient(180deg, #fffaf4 0%, #fff0df 100%);
        }

        body {
            font-family: "Trebuchet MS", "Avenir Next", "Segoe UI", sans-serif;
            color: var(--ink);
        }

        button,
        input,
        select {
            font: inherit;
        }

        button {
            cursor: pointer;
        }

        img {
            display: block;
        }

        #app,
        body {
            min-height: 100vh;
        }

        .app-shell {
            position: relative;
            width: 100%;
            max-width: 440px;
            min-height: 100vh;
            margin: 0 auto;
            background:
                radial-gradient(circle at top, rgba(255, 255, 255, 0.88), transparent 38%),
                linear-gradient(180deg, rgba(255, 247, 236, 0.96), rgba(255, 239, 221, 0.96));
            box-shadow: var(--shadow);
            overflow: hidden;
        }

        .content {
            min-height: 100vh;
            padding-bottom: 106px;
        }

        .page {
            min-height: 100vh;
            padding: 24px 20px 24px;
        }

        .page-home {
            padding-top: 20px;
        }

        .hero-card,
        .panel,
        .settings-panel,
        .success-panel {
            background: var(--panel);
            backdrop-filter: blur(16px);
            border: 1px solid var(--border);
            border-radius: 28px;
            box-shadow: 0 14px 32px rgba(94, 66, 38, 0.08);
        }

        .hero-card {
            padding: 20px;
            display: grid;
            grid-template-columns: 1.2fr 0.9fr;
            gap: 18px;
            align-items: center;
            margin-bottom: 18px;
        }

        .hero-card-single {
            grid-template-columns: 1fr;
        }

        .hero-copy {
            display: flex;
            flex-direction: column;
            gap: 10px;
        }

        .eyebrow {
            font-size: 12px;
            letter-spacing: 0.18em;
            text-transform: uppercase;
            color: var(--primary-dark);
            font-weight: 700;
        }

        .hero-title,
        .success-title {
            font-family: "Georgia", "Times New Roman", serif;
            font-size: 36px;
            line-height: 1.05;
            font-weight: 700;
        }

        .hero-subtitle,
        .panel-subtitle,
        .cta-copy,
        .settings-hint,
        .success-copy {
            color: var(--muted);
            font-size: 14px;
            line-height: 1.45;
        }

        .home-avatar,
        .settings-photo-frame {
            aspect-ratio: 1;
            border-radius: 50%;
            overflow: hidden;
            background: linear-gradient(180deg, #f8deba, #efbf8b);
            border: 4px solid rgba(255, 255, 255, 0.85);
            box-shadow: 0 10px 28px rgba(94, 66, 38, 0.16);
        }

        .home-avatar {
            width: 132px;
            justify-self: end;
        }

        .home-avatar-image,
        .settings-photo {
            width: 100%;
            height: 100%;
            object-fit: cover;
        }

        .status-strip,
        .metrics-grid,
        .cta-grid,
        .schedule-editor,
        .schedule-preview,
        .settings-panel,
        .success-actions {
            display: grid;
            gap: 14px;
        }

        .status-strip {
            grid-template-columns: repeat(3, minmax(0, 1fr));
            margin-bottom: 18px;
        }

        .status-chip,
        .metric-card,
        .schedule-row,
        .schedule-card,
        .settings-field,
        .settings-action {
            background: var(--panel-strong);
            border: 1px solid var(--border);
            border-radius: 22px;
        }

        .status-chip,
        .metric-card {
            padding: 14px;
        }

        .chip-label,
        .metric-label,
        .portion-caption {
            display: block;
            font-size: 12px;
            text-transform: uppercase;
            letter-spacing: 0.12em;
            color: var(--muted);
            margin-bottom: 8px;
            font-weight: 700;
        }

        .chip-value,
        .metric-value {
            display: block;
            font-size: 18px;
            line-height: 1.25;
            font-weight: 700;
        }

        .chip-value.online {
            color: var(--success);
        }

        .chip-value.offline,
        .settings-tag.warning {
            color: var(--offline);
        }

        .chip-value.warm {
            color: var(--warning);
        }

        .chip-value.muted {
            color: var(--muted);
        }

        .metrics-grid,
        .cta-grid {
            grid-template-columns: 1fr;
            margin-bottom: 18px;
        }

        .cta-button,
        .feed-now-button,
        .text-button,
        .toggle-button,
        .stepper-button,
        .portion-btn,
        .back-button,
        .nav-btn {
            border: none;
            transition: transform 140ms ease, background 140ms ease, opacity 140ms ease;
        }

        .cta-button:hover,
        .feed-now-button:hover,
        .text-button:hover,
        .toggle-button:hover,
        .stepper-button:hover,
        .portion-btn:hover,
        .nav-btn:hover {
            transform: translateY(-1px);
        }

        .cta-button {
            padding: 18px;
            text-align: left;
            border-radius: 24px;
            display: flex;
            flex-direction: column;
            gap: 6px;
        }

        .cta-primary,
        .feed-now-button {
            color: #fff9f3;
            background: linear-gradient(135deg, var(--primary) 0%, #ff8d45 100%);
            box-shadow: 0 16px 24px rgba(242, 106, 61, 0.24);
        }

        .cta-secondary {
            background: linear-gradient(135deg, #ffe5c6 0%, #fff5e8 100%);
            color: var(--ink);
        }

        .cta-title,
        .panel-title,
        .app-title,
        .settings-label {
            font-size: 18px;
            font-weight: 700;
        }

        .panel {
            padding: 18px;
        }

        .panel-tight {
            padding: 18px;
        }

        .panel-header,
        .schedule-row,
        .schedule-card-head,
        .schedule-portion-row,
        .settings-action,
        .top-bar {
            display: flex;
            align-items: center;
            justify-content: space-between;
            gap: 12px;
        }

        .text-button {
            background: transparent;
            color: var(--primary-dark);
            font-weight: 700;
        }

        .schedule-row,
        .schedule-card,
        .settings-field,
        .settings-action {
            padding: 16px;
        }

        .schedule-label {
            font-size: 16px;
            font-weight: 700;
            margin-bottom: 4px;
        }

        .schedule-detail {
            font-size: 14px;
            color: var(--muted);
        }

        .pill-badge,
        .settings-tag,
        .toggle-button {
            padding: 8px 12px;
            border-radius: 999px;
            background: #f8ecdc;
            color: var(--muted);
            font-size: 13px;
            font-weight: 700;
        }

        .pill-badge.active,
        .settings-tag.enabled,
        .toggle-button.enabled {
            background: rgba(63, 155, 104, 0.12);
            color: var(--success);
        }

        .pill-badge.inactive {
            background: rgba(178, 90, 98, 0.1);
            color: var(--offline);
        }

        .top-bar {
            position: relative;
            justify-content: center;
            min-height: 46px;
            margin-bottom: 18px;
        }

        .back-button {
            position: absolute;
            left: 0;
            width: 42px;
            height: 42px;
            border-radius: 50%;
            background: rgba(255, 255, 255, 0.75);
            color: var(--ink);
            font-size: 24px;
        }

        .feed-controls {
            display: grid;
            grid-template-columns: 64px 1fr 64px;
            align-items: center;
            gap: 12px;
            margin: 18px 0 8px;
        }

        .portion-btn,
        .stepper-button {
            width: 100%;
            min-height: 52px;
            border-radius: 18px;
            background: #fff0de;
            color: var(--ink);
            font-size: 32px;
        }

        .portion-value {
            text-align: center;
            font-family: "Georgia", "Times New Roman", serif;
            font-size: 80px;
            line-height: 1;
            color: var(--primary-dark);
        }

        .portion-helper {
            text-align: center;
            font-size: 16px;
            color: var(--muted);
            margin-bottom: 10px;
        }

        .pills-row {
            min-height: 148px;
            display: flex;
            align-items: center;
            justify-content: center;
            margin-bottom: 16px;
        }

        .pills-row img {
            height: auto;
        }

        .pills-size-1 {
            width: 140px;
        }

        .pills-size-2 {
            width: 205px;
        }

        .pills-size-3 {
            width: 265px;
        }

        .feed-now-button {
            width: 100%;
            min-height: 62px;
            border-radius: 22px;
            font-size: 18px;
            font-weight: 700;
        }

        .schedule-editor,
        .settings-panel,
        .success-actions {
            margin-top: 14px;
        }

        .schedule-form,
        .schedule-actions,
        .portion-picker,
        .setup-list {
            display: grid;
            gap: 14px;
        }

        .setup-card {
            width: 100%;
            min-height: 76px;
            padding: 16px;
            display: flex;
            align-items: center;
            justify-content: space-between;
            gap: 12px;
            text-align: left;
            border: 1px solid var(--border);
            border-radius: 22px;
            background: var(--panel-strong);
            color: var(--ink);
        }

        .setup-card-title {
            font-size: 17px;
            font-weight: 700;
            margin-bottom: 4px;
        }

        .setup-card-copy {
            color: var(--muted);
            font-size: 14px;
            line-height: 1.4;
        }

        .setup-main-action {
            margin-top: 16px;
        }

        .setup-loader {
            min-height: 180px;
            display: grid;
            place-items: center;
            align-content: center;
            gap: 14px;
        }

        .setup-spinner {
            width: 42px;
            height: 42px;
            border-radius: 50%;
            border: 4px solid rgba(242, 106, 61, 0.18);
            border-top-color: var(--primary);
            animation: setup-spin 900ms linear infinite;
        }

        .setup-modal-backdrop {
            position: fixed;
            inset: 0;
            z-index: 20;
            display: flex;
            align-items: flex-end;
            justify-content: center;
            padding: 18px;
            background: rgba(45, 33, 24, 0.28);
        }

        .setup-modal {
            width: 100%;
            max-width: 404px;
            padding: 18px;
            display: grid;
            gap: 14px;
            border-radius: 24px;
            border: 1px solid var(--border);
            background: var(--panel-strong);
            box-shadow: var(--shadow);
        }

        .setup-cancel {
            justify-self: center;
        }

        @keyframes setup-spin {
            to {
                transform: rotate(360deg);
            }
        }

        .schedule-card {
            display: grid;
            gap: 16px;
        }

        .schedule-portion-row {
            align-items: center;
        }

        .portion-stepper {
            display: flex;
            align-items: center;
            gap: 10px;
        }

        .portion-inline-value {
            min-width: 88px;
            text-align: center;
            font-weight: 700;
        }

        .secondary-inline-button {
            min-height: 44px;
            border-radius: 16px;
            background: #fff0de;
            color: var(--ink);
            font-weight: 700;
            border: none;
        }

        .secondary-inline-button.danger {
            background: rgba(178, 90, 98, 0.12);
            color: var(--offline);
        }

        .empty-state {
            padding: 18px;
            border-radius: 22px;
            background: rgba(255, 255, 255, 0.68);
            border: 1px dashed rgba(94, 66, 38, 0.18);
            display: grid;
            gap: 6px;
        }

        .empty-title {
            font-size: 16px;
            font-weight: 700;
        }

        .empty-copy,
        .inline-status {
            color: var(--muted);
            line-height: 1.45;
        }

        .inline-status {
            margin-top: 14px;
            font-size: 14px;
        }

        .schedule-list-panel {
            margin-top: 16px;
        }

        .feed-now-button:disabled {
            opacity: 0.7;
            cursor: wait;
            transform: none;
        }

        .hero-panel {
            margin-bottom: 16px;
        }

        .illustration-wrap {
            display: flex;
            justify-content: center;
            margin-bottom: 12px;
        }

        .illustration-wrap img {
            width: 100%;
            max-width: 220px;
            height: auto;
        }

        .timeline-list {
            display: grid;
            gap: 14px;
        }

        .timeline-item {
            display: grid;
            grid-template-columns: 24px 1fr;
            gap: 12px;
            align-items: stretch;
        }

        .timeline-side {
            display: flex;
            flex-direction: column;
            align-items: center;
            gap: 6px;
        }

        .timeline-dot {
            width: 14px;
            height: 14px;
            border-radius: 50%;
            background: var(--warning);
            box-shadow: 0 0 0 6px rgba(255, 179, 77, 0.16);
        }

        .timeline-dot.success {
            background: var(--success);
            box-shadow: 0 0 0 6px rgba(63, 155, 104, 0.14);
        }

        .timeline-dot.warning {
            background: var(--warning);
        }

        .timeline-dot.info {
            background: var(--primary);
            box-shadow: 0 0 0 6px rgba(242, 106, 61, 0.14);
        }

        .timeline-line {
            width: 2px;
            flex: 1;
            background: linear-gradient(180deg, rgba(242, 106, 61, 0.44), rgba(242, 106, 61, 0));
            border-radius: 999px;
        }

        .timeline-card {
            background: rgba(255, 255, 255, 0.78);
            border: 1px solid var(--border);
            border-radius: 22px;
            padding: 16px;
        }

        .timeline-card-header {
            display: flex;
            align-items: baseline;
            justify-content: space-between;
            gap: 12px;
            margin-bottom: 8px;
            font-size: 14px;
        }

        .timeline-card p,
        .timeline-card span {
            color: var(--muted);
            line-height: 1.45;
        }

        .settings-hero {
            display: flex;
            justify-content: center;
            margin-bottom: 18px;
        }

        .settings-photo-frame {
            width: 168px;
        }

        .settings-field {
            display: grid;
            gap: 8px;
        }

        .settings-input {
            border: 1px solid rgba(94, 66, 38, 0.16);
            border-radius: 16px;
            padding: 12px 14px;
            background: #fff9f2;
            color: var(--ink);
        }

        .settings-action {
            width: 100%;
            text-align: left;
            background: var(--panel-strong);
        }

        .settings-action > div {
            display: grid;
            gap: 4px;
        }

        .settings-arrow {
            font-size: 24px;
            color: var(--primary-dark);
        }

        .success-panel {
            padding: 24px;
            text-align: center;
            display: grid;
            gap: 14px;
        }

        .hidden-input {
            display: none;
        }

        .bottom-navigation {
            position: fixed;
            left: 50%;
            bottom: 0;
            transform: translateX(-50%);
            width: 100%;
            max-width: 440px;
            height: 88px;
            background: rgba(255, 251, 245, 0.88);
            backdrop-filter: blur(16px);
            border-top: 1px solid var(--border);
            display: flex;
            align-items: center;
            justify-content: space-around;
            padding: 0 26px;
        }

        .nav-btn {
            width: 58px;
            height: 58px;
            border-radius: 18px;
            background: transparent;
            opacity: 0.66;
        }

        .nav-btn.active {
            background: rgba(242, 106, 61, 0.12);
            opacity: 1;
        }

        .nav-icon {
            width: 30px;
            height: 30px;
            object-fit: contain;
            margin: 0 auto;
        }

        @media (max-width: 380px) {
            .hero-card {
                grid-template-columns: 1fr;
            }

            .home-avatar {
                justify-self: center;
            }

            .status-strip {
                grid-template-columns: 1fr;
            }

            .feed-controls {
                grid-template-columns: 56px 1fr 56px;
            }

            .portion-value {
                font-size: 64px;
            }
        }
    "#;
}
