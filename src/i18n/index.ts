import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';
import zhCN from './zh-CN.json';
import en from './en.json';

const resources = {
  'zh-CN': { translation: zhCN },
  en: { translation: en },
};

// Detect browser language
const getBrowserLanguage = (): string => {
  const lang = navigator.language;
  if (lang.startsWith('zh')) return 'zh-CN';
  return 'en';
};

i18n.use(initReactI18next).init({
  resources,
  lng: getBrowserLanguage(),
  fallbackLng: 'en',
  interpolation: {
    escapeValue: false,
  },
});

export default i18n;
