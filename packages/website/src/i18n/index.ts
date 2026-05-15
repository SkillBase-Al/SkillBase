import type { AstroGlobal } from 'astro';
import zh from './zh.json';
import en from './en.json';

const translations = { zh, en } as const;
type Locale = keyof typeof translations;
type TranslationDict = typeof zh;

export function getLocale(url: URL): Locale {
  if (url.pathname.startsWith('/en/') || url.pathname === '/en') {
    return 'en';
  }
  return 'zh';
}

export function useTranslations(locale: Locale) {
  const dict = translations[locale] as TranslationDict;

  // Simple dot-path accessor for nested keys
  function t(path: string): string {
    return path.split('.').reduce((obj: unknown, key: string) => {
      if (obj && typeof obj === 'object' && key in obj) {
        return (obj as Record<string, unknown>)[key];
      }
      return path; // fallback to the path key
    }, dict) as string;
  }

  return { t, locale, dict };
}
