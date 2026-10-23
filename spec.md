# Especificação do Projeto: Fedplay

**Versão**: 1.2  
**Data**: 23 de outubro de 2026  
**Status**: Decisões Técnicas Definidas  
**Metodologia**: SDD (Specification-Driven Development)

---

## 1. Visão Geral do Projeto

### 1.1. Contexto

O **Fedplay** é um reprodutor de áudio nativo para o ecossistema Fedora Linux, projetado para preencher a lacuna entre reprodutores minimalistas (que carecem de recursos avançados) e reprodutores completos (que são pesados e complexos). O projeto visa oferecer uma experiência de reprodução de áudio elegante, leve e integrada ao sistema, com foco especial na gestão de metadados.

### 1.2. Público-Alvo

- Usuários do Fedora Linux que valorizam integração nativa com o sistema.
- Entusiastas de áudio que desejam organizar suas bibliotecas locais sem recorrer a soluções pesadas.
- Desenvolvedores e contribuidores da comunidade open-source interessadas em aplicações GTK/Qt modernas.

### 1.3. Proposta de Valor

- **Leveza**: Consumo mínimo de recursos (CPU/RAM).
- **Integração Nativa**: Segue as diretrizes de design do Fedora (GTK4/Libadwaita).
- **Foco em Metadados**: Edição robusta de tags (ID3, MP4) como funcionalidade central.
- **Simplicidade**: Interface limpa, sem funcionalidades desnecessárias.
- **Segurança**: Desenvolvido em Rust para garantir segurança de memória.

---

## 2. Escopo do Projeto

### 2.1. Dentro do Escopo (In-Scope)

- Reprodução de arquivos de áudio locais nos formatos MP3 e M4A (AAC).
- Visualização e edição de metadados (tags) dos arquivos.
- Gerenciamento de listas de reprodução (playlists).
- Integração com o sistema de notificações do Fedora (MPRIS2).
- Empacotamento em RPM nativo e Flatpak.

### 2.2. Fora do Escopo (Out-of-Scope)

- Streaming de serviços online (Spotify, YouTube Music, etc.).
- Reprodução de formatos de alta resolução (FLAC, DSD, ALAC) — _podem ser considerados em versões futuras_.
- Sincronização com dispositivos móveis.
- Equalizador avançado ou efeitos DSP.
- Suporte a Windows/macOS (foco exclusivo em Linux/Fedora).

---

## 3. Requisitos Funcionais (RF)

| ID   | Requisito                                                                                                    | Prioridade | Critério de Aceitação                                                                                 |
| ---- | ------------------------------------------------------------------------------------------------------------ | ---------- | ----------------------------------------------------------------------------------------------------- |
| RF01 | O sistema deve reproduzir arquivos nos formatos **MP3** e **M4A (AAC)**.                                     | Alta       | Reprodução sem interrupções, com suporte a taxas de amostragem de 8kHz a 48kHz.                       |
| RF02 | O sistema deve exibir metadados do arquivo em reprodução (título, artista, álbum, capa, faixa, ano, gênero). | Alta       | Todas as tags suportadas devem ser exibidas na interface em até 200ms após o início da reprodução.    |
| RF03 | O sistema deve permitir a **edição de metadados** (ID3v2 para MP3, átomos MP4 para M4A).                     | Alta       | Alterações devem ser salvas permanentemente no arquivo, com opção de desfazer/refazer.                |
| RF04 | O sistema deve permitir a extração e incorporação de **capas de álbum** (embedded art).                      | Média      | Suporte a imagens JPEG/PNG até 5MB, redimensionamento automático para padrões comuns (ex: 500x500px). |
| RF05 | O sistema deve gerenciar **listas de reprodução** (criar, editar, excluir, reordenar).                       | Alta       | Playlists devem ser salvas em formato M3U8 ou formato proprietário baseado em SQLite.                 |
| RF06 | O sistema deve integrar-se ao **D-Bus MPRIS2** para controle externo (teclas multimídia, applets de painel). | Alta       | Controles de play/pause/next/prev devem funcionar via teclado e applets do GNOME/KDE.                 |
| RF07 | O sistema deve suportar **arrastar e soltar** de arquivos/pastas para a biblioteca.                          | Média      | Aceitar múltiplos arquivos e pastas recursivamente.                                                   |
| RF08 | O sistema deve oferecer **busca** na biblioteca por título, artista, álbum ou gênero.                        | Alta       | Resultados exibidos em tempo real (enquanto o usuário digita).                                        |
| RF09 | O sistema deve manter **histórico de reprodução** e marcar faixas como favoritas.                            | Baixa      | Dados persistidos entre sessões.                                                                      |
| RF10 | O sistema deve oferecer **modo escuro/claro** seguindo o tema do sistema.                                    | Alta       | Detecção automática via portal XDG.                                                                   |

---

## 4. Requisitos Não Funcionais (RNF)

| ID    | Requisito                               | Métrica                                                                                                  |
| ----- | --------------------------------------- | -------------------------------------------------------------------------------------------------------- |
| RNF01 | **Tempo de inicialização** (cold start) | ≤ 1.5 segundos em hardware de referência (CPU dual-core, 4GB RAM, SSD).                                  |
| RNF02 | **Consumo de memória RAM** em repouso   | ≤ 50 MB (após carregamento da biblioteca).                                                               |
| RNF03 | **Consumo de CPU** durante reprodução   | ≤ 3% em idle de reprodução (hardware de referência).                                                     |
| RNF04 | **Tempo de resposta da interface**      | ≤ 100ms para interações do usuário (botões, menus).                                                      |
| RNF05 | **Suporte a Wayland**                   | 100% compatível, sem dependência de X11.                                                                 |
| RNF06 | **Acessibilidade**                      | Conformidade com WCAG 2.1 nível AA; navegação completa por teclado; suporte a leitores de tela (AT-SPI). |
| RNF07 | **Internacionalização (i18n)**          | Suporte a múltiplos idiomas via gettext; inglês e português brasileiro como idiomas base.                |
| RNF08 | **Segurança**                           | Nenhum dado do usuário deve ser transmitido pela rede (aplicação 100% offline).                          |
| RNF09 | **Confiabilidade**                      | Taxa de falhas (crash) inferior a 0.1% em uso contínuo de 24h.                                           |
| RNF10 | **Manutenibilidade**                    | Cobertura de testes unitários ≥ 70%; documentação técnica em Markdown.                                   |

---

## 5. Requisitos de Interface (UI/UX)

### 5.1. Diretrizes de Design

- Seguir o **GNOME Human Interface Guidelines (HIG)**.
- Utilizar **Libadwaita** para componentes nativos e consistentes com o Fedora.
- Layout adaptativo: funcional em telas de 1024x768 até 4K.

### 5.2. Estrutura de Telas (Wireframes Conceituais)

1. **Tela Principal**: Lista de reprodução atual + controles de reprodução + visualização de metadados.
2. **Tela de Biblioteca**: Grade/lista de faixas com filtros e busca.
3. **Tela de Edição de Metadados**: Formulário modal com campos editáveis e preview da capa.
4. **Tela de Preferências**: Configurações de comportamento, atalhos e aparência.

### 5.3. Atalhos de Teclado (Obrigatórios)

- `Espaço`: Play/Pause
- `Ctrl+→`: Próxima faixa
- `Ctrl+←`: Faixa anterior
- `Ctrl+F`: Buscar
- `Ctrl+E`: Editar metadados da faixa selecionada
- `Ctrl+,`: Abrir preferências

---

## 6. Requisitos de Dados

### 6.1. Formatos Suportados

| Formato | Container      | Codec                  | Metadados                           |
| ------- | -------------- | ---------------------- | ----------------------------------- |
| MP3     | -              | MPEG-1/2 Audio Layer 3 | ID3v1, ID3v2.3, ID3v2.4             |
| M4A     | MPEG-4 Part 14 | AAC-LC, HE-AAC         | Átomos MP4 (©nam, ©ART, ©alb, covr) |

### 6.2. Estrutura de Persistência

- **Biblioteca**: Banco de dados SQLite local (`~/.local/share/fedplay/library.db`).
- **Playlists**: Arquivos M3U8 em `~/.local/share/fedplay/playlists/`.
- **Configurações**: GSettings em `~/.config/fedplay/`.
- **Cache de capas**: `~/.cache/fedplay/covers/`.

---

## 7. Requisitos de Integração com o Sistema

- **D-Bus**: Expor interface MPRIS2 em `org.mpris.MediaPlayer2.Fedplay`.
- **Portais XDG**: Usar `org.freedesktop.portal.FileChooser` para seleção de arquivos.
- **Notificações**: Integrar com `org.freedesktop.Notifications` para alertas.
- **Temas**: Respeitar `org.freedesktop.appearance.color-scheme` (dark/light).

---

## 8. Requisitos de Empacotamento e Distribuição

| Formato        | Prioridade | Observações                                                |
| -------------- | ---------- | ---------------------------------------------------------- |
| **RPM nativo** | Alta       | Disponível no Fedora via Copr ou submissão oficial.        |
| **Flatpak**    | Alta       | Publicado no Flathub para distribuição universal em Linux. |
| **AppImage**   | Baixa      | Opcional, para usuários de outras distros.                 |

---

## 9. Critérios de Aceitação Globais (Definition of Done)

Um requisito é considerado "pronto" quando:

1. Implementado conforme especificado.
2. Coberto por testes unitários e de integração.
3. Documentado (código + manual do usuário).
4. Revisado por pelo menos um outro desenvolvedor (code review).
5. Testado manualmente em ambiente Fedora (Wayland, GNOME).
6. Empacotado e testado em RPM e Flatpak.

---

## 10. Restrições e Dependências

### 10.1. Restrições

- Licença: **GPLv3** (para compatibilidade com o ecossistema Fedora).
- Linguagem de desenvolvimento: **Rust**.
- Sem dependências de bibliotecas proprietárias.

### 10.2. Stack Tecnológica (Definida)

| Componente              | Tecnologia    | Versão Mínima | Justificativa                                          |
| ----------------------- | ------------- | ------------- | ------------------------------------------------------ |
| **Linguagem**           | Rust          | 1.70+         | Segurança de memória, performance, ecossistema moderno |
| **UI Framework**        | GTK4          | 4.10+         | Integração nativa com GNOME/Fedora                     |
| **UI Toolkit**          | Libadwaita    | 1.3+          | Design system Adwaita, componentes modernos            |
| **Bindings GTK**        | gtk4-rs       | 0.7+          | Bindings Rust para GTK4                                |
| **Bindings Libadwaita** | libadwaita-rs | 0.5+          | Bindings Rust para Libadwaita                          |
| **Backend de Áudio**    | GStreamer     | 1.20+         | Padrão GNOME, pipeline-based, flexível                 |
| **Bindings GStreamer**  | gstreamer-rs  | 0.21+         | Bindings Rust para GStreamer                           |
| **Metadados**           | lofty-rs      | 0.16+         | Manipulação nativa de tags em Rust, sem FFI            |
| **Banco de Dados**      | rusqlite      | 0.31+         | SQLite para biblioteca local                           |
| **Async Runtime**       | tokio         | 1.32+         | Assíncrono para I/O e operações de rede (futuro)       |
| **D-Bus**               | zbus          | 3.14+         | Bindings Rust para D-Bus (MPRIS2)                      |

---

## 11. Arquitetura do Sistema

### 11.1. Diagrama de Arquitetura (Alto Nível)
