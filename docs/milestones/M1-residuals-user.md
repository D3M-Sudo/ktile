# Protocollo Residui M1 (B5 e B6) sulla Macchina Target

> **Scopo del documento**: Questo protocollo contiene le istruzioni ed i comandi esatti da eseguire sulla tua macchina fisica Linux Mint (Cinnamon / Muffin X11) per completare le uniche due verifiche residue che richiedono la sessione grafica reale.
> **Tempo stimato**: circa 10 minuti.

---

## Residuo B5 — Osservazione sequenza Unmap / MapNotify durante Minimizzazione

### Obiettivo
Verificare se la minimizzazione di una finestra su Muffin genera eventi di `UnmapNotify` sul server X11 o se imposta esclusivamente la proprietà `_NET_WM_STATE_HIDDEN` (fondamentale per la decisione D8 di ripristino slot).

### Procedura (3 minuti)

1. **Apri una finestra campione** (ad es. Nemo o un terminale) e selezionala.
2. **Apri un terminale** e rileva l'ID della finestra (WID) con il comando:
   ```bash
   WID=$(xdotool getactivewindow)
   echo "WID rilevato: $WID"
   ```
3. **Avvia l'osservatore di eventi** con il seguente comando one-liner:
   ```bash
   (
     echo "=== T0: Stato Iniziale ==="
     xprop -id $WID _NET_WM_STATE
     echo "=== Inizio osservazione (minimizzare la finestra ORA) ==="
     for i in $(seq 1 6); do
       sleep 1
       echo "--- t=$i s ---"
       xprop -id $WID _NET_WM_STATE
     done
     echo "=== CLIENT LIST finale ==="
     xprop -root _NET_CLIENT_LIST
   )
   ```
4. **Azione durante il loop**: Minimizza la finestra dalla barra delle applicazioni entro 2 secondi dall'avvio dello script, poi ripristinala al secondo 5.
5. **Cosa incollare**: Copia e incolla il testo di output prodotto dal terminale.

---

## Residuo B6 — Verifica `_NET_WORKAREA` e Strut con Pannello Spostato o Auto-Hide

> **AVVISO CONSENSO**: Questo test richiede la modifica temporanea delle impostazioni del pannello di Cinnamon. Esegui questo punto **solo se dai il tuo consenso** a spostare temporaneamente il pannello o attivare l'auto-hide. Il ripristino richiede 30 secondi.

### Obiettivo
Verificare come Muffin aggiorna la proprietà `_NET_WORKAREA` quando il pannello di Cinnamon viene spostato (es. a sinistra) oppure impostato su "nascondi automaticamente" (ADR-12).

### Procedura (5 minuti)

1. **Stato iniziale (Pannello in basso - Baseline)**:
   Esegui nel terminale:
   ```bash
   echo "=== 1. BASELINE (Pannello in basso) ==="
   xprop -root _NET_WORKAREA
   ```

2. **Modifica configurazione Pannello**:
   - Fai **tasto destro** su un punto vuoto del pannello di Cinnamon.
   - Seleziona **Impostazioni del pannello** (Panel settings).
   - *Opzione A (Consigliata)*: Cambia la modalità **Nascondi automaticamente** (Auto-hide) da "Mai" a "Nascondi sempre" (o "Nascondi intelligentemente").
   - *Opzione B*: Seleziona **Sposta pannello** e trascinalo sul bordo **sinistro** o **superiore** dello schermo.

3. **Verifica aggiornamento Workarea**:
   Esegui immediatamente nel terminale:
   ```bash
   echo "=== 2. DOPO MODIFICA PANNELLO (Auto-hide o Spostato) ==="
   xprop -root _NET_WORKAREA
   ```

4. **Ripristino Impostazioni**:
   - Riporta la modalità **Nascondi automaticamente** su "Mai" (o risposta il pannello in basso).
   - Esegui per confermare il ripristino:
     ```bash
     echo "=== 3. REVERT CONFERMATO (Pannello in basso) ==="
     xprop -root _NET_WORKAREA
     ```

5. **Cosa incollare**: Copia e incolla i tre blocchi di output generati dai comandi `xprop -root _NET_WORKAREA`.

---

## Riepilogo degli Output da Incollare
Quando completato, puoi incollare in un commento o messaggio gli output dei due comandi sopra indicati.
