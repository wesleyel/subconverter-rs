import { getUserLocale } from '@/services/locale';
import { getRequestConfig } from 'next-intl/server';

// Supported languages
export const locales = ['en', 'zh'] as const;
export type Locale = (typeof locales)[number];

export default getRequestConfig(async () => {

    const locale = await getUserLocale();

    return {
        locale: locale,
        // Load messages for the resolved locale
        messages: (await import(`../../messages/${locale}.json`)).default
    };
});

// Helper function to extract locale from Accept-Language header
function getLocaleFromHeader(acceptLanguage?: string | null): string | undefined {
    if (!acceptLanguage) return undefined;

    // Parse the Accept-Language header
    const languages = acceptLanguage
        .split(',')
        .map(lang => {
            const [code, weight] = lang.trim().split(';q=');
            return {
                code: code.split('-')[0], // Get primary language code
                weight: weight ? Number(weight) : 1.0
            };
        })
        .sort((a, b) => b.weight - a.weight); // Sort by weight (highest first)

    // Find the first supported language
    const match = languages.find(lang => locales.includes(lang.code as Locale));
    return match?.code;
} 