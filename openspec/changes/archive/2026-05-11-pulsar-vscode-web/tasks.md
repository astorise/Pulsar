# Implementation Tasks

- [x] **Task 1: Scaffold Web Extension**
  - Initialiser une extension VS Code nommée `pulsar-vscode`.
  - Configurer impérativement comme une **Web Extension** (pour exécution dans le host navigateur).
  - Ajouter les dépendances : `esbuild`, `@types/vscode`.

- [x] **Task 2: Define Contributions**
  - Ajouter les propriétés de configuration `pulsar.*` dans le `package.json`.
  - Enregistrer le `viewsContainers` (icône Pulsar) et la `views` pour le chat.

- [x] **Task 3: Implement Webview Provider**
  - Créer `src/ChatViewProvider.ts`.
  - Implémenter l'interface `vscode.WebviewViewProvider` pour le rendu HTML/JS du chat.

- [x] **Task 4: Implement WebSocket Client**
  - Créer `src/PulsarClient.ts`.
  - Utiliser l'API native `WebSocket` (compatible Web Extension).
  - Gérer le handshake `Init` et le streaming des tokens.

- [x] **Task 5: Editor Integration**
  - Implémenter la commande `pulsar.sendSelection` pour capturer le code surligné.

- [x] **Task 6: Update CI/CD Pipeline**
  - Modifier `.github/workflows/build.yml` pour ajouter le job `build-vscode-extension`.
  - Installer `@vscode/vsce` dans l'environnement de build.
  - Configurer le packaging avec le flag `--target web` pour générer le fichier `.vsix`.
  - Ajouter l'étape `upload-artifact` pour le fichier `pulsar-web-extension.vsix`.
