/******************************  Quersumme.java  *************************/

import AlgoTools.IO;

/**
 * Liest eine Zahl >= 0 ein und gibt die Quersumme der Zahl aus.
 */

public class Quersumme {
  public static void main(String[] args) {
    int zahl;

    do {
      zahl = IO.readInt("Zahl > 0 zur Berechnung Quersumme eingeben: ");
    } while(zahl <= 0);

    int quersumme = 0;

    // So lange Stellen abschneiden und addieren, bis keine mehr
    // da sind.
    while(zahl > 0) {
      // letzte Stelle aufaddieren
      quersumme = quersumme + zahl % 10;

      // letzte Stelle durch ganzzahliges Teilen abschneiden
      zahl = zahl / 10;
    }

    IO.println("Die Quersumme ist: " + quersumme);
  }
}
