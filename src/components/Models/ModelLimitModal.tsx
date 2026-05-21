import { useEffect, useMemo, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { AlertCircle, Gauge, Loader2 } from 'lucide-react';
import { Modal } from '../common/Modal';
import { Button } from '../common/Button';
import {
  getCustomModelMetadata,
  updateCustomModelLimit,
  type ModelLimit,
} from '../../services/tauri';

interface ModelLimitModalProps {
  isOpen: boolean;
  onClose: () => void;
  providerId: string;
  modelId: string;
  onSaved?: () => void;
}

type LimitForm = {
  context: string;
  input: string;
  output: string;
};

const emptyForm: LimitForm = {
  context: '',
  input: '',
  output: '',
};

function parsePositiveInteger(value: string): number | null {
  if (!/^\d+$/.test(value.trim())) {
    return null;
  }

  const parsed = Number(value);
  if (!Number.isSafeInteger(parsed) || parsed <= 0) {
    return null;
  }

  return parsed;
}

function errorMessage(error: unknown, fallback: string): string {
  if (error instanceof Error) {
    return error.message;
  }
  if (typeof error === 'string') {
    return error;
  }
  return fallback;
}

export function ModelLimitModal({
  isOpen,
  onClose,
  providerId,
  modelId,
  onSaved,
}: ModelLimitModalProps) {
  const { t } = useTranslation();
  const [form, setForm] = useState<LimitForm>(emptyForm);
  const [isLoading, setIsLoading] = useState(false);
  const [isSaving, setIsSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!isOpen) {
      return;
    }

    let isCurrent = true;
    setIsLoading(true);
    setError(null);
    setForm(emptyForm);

    getCustomModelMetadata(providerId, modelId)
      .then((metadata) => {
        if (!isCurrent) {
          return;
        }

        const limit = metadata.limit;
        setForm({
          context: limit?.context ? String(limit.context) : '',
          input: limit?.input ? String(limit.input) : '',
          output: limit?.output ? String(limit.output) : '',
        });
      })
      .catch((err) => {
        if (!isCurrent) {
          return;
        }
        setError(errorMessage(err, t('modelLimit.loadError')));
      })
      .finally(() => {
        if (isCurrent) {
          setIsLoading(false);
        }
      });

    return () => {
      isCurrent = false;
    };
  }, [isOpen, modelId, providerId, t]);

  const validationError = useMemo(() => {
    if (!form.context.trim()) {
      return t('modelLimit.contextRequired');
    }
    if (parsePositiveInteger(form.context) === null) {
      return t('modelLimit.contextInvalid');
    }
    if (form.input.trim() && parsePositiveInteger(form.input) === null) {
      return t('modelLimit.inputInvalid');
    }
    if (!form.output.trim()) {
      return t('modelLimit.outputRequired');
    }
    if (parsePositiveInteger(form.output) === null) {
      return t('modelLimit.outputInvalid');
    }
    return null;
  }, [form.context, form.input, form.output, t]);

  const handleFieldChange = (field: keyof LimitForm, value: string) => {
    setForm((current) => ({ ...current, [field]: value }));
    setError(null);
  };

  const handleSave = async () => {
    const context = parsePositiveInteger(form.context);
    const input = form.input.trim() ? parsePositiveInteger(form.input) : null;
    const output = parsePositiveInteger(form.output);

    if (context === null || output === null || (form.input.trim() && input === null)) {
      setError(validationError ?? t('modelLimit.invalidForm'));
      return;
    }

    const limit: ModelLimit = {
      context,
      output,
    };
    if (input !== null) {
      limit.input = input;
    }

    try {
      setIsSaving(true);
      setError(null);
      await updateCustomModelLimit(providerId, modelId, limit);
      onSaved?.();
      onClose();
    } catch (err) {
      setError(errorMessage(err, t('modelLimit.saveError')));
    } finally {
      setIsSaving(false);
    }
  };

  return (
    <Modal
      isOpen={isOpen}
      onClose={onClose}
      title={t('modelLimit.title')}
      size="md"
      footer={
        <>
          <Button variant="ghost" onClick={onClose} disabled={isSaving}>
            {t('button.cancel')}
          </Button>
          <Button
            onClick={handleSave}
            disabled={isLoading || Boolean(validationError)}
            isLoading={isSaving}
          >
            {t('button.save')}
          </Button>
        </>
      }
    >
      <div className="space-y-4">
        <div className="flex items-start gap-3 rounded-xl border border-indigo-100 bg-indigo-50/70 p-3">
          <div className="mt-0.5 flex h-8 w-8 flex-shrink-0 items-center justify-center rounded-lg bg-white text-indigo-600 shadow-sm">
            <Gauge className="h-4 w-4" />
          </div>
          <div className="min-w-0">
            <p className="text-sm font-semibold text-slate-800">{modelId}</p>
            <p className="text-xs text-slate-500">
              {providerId} · {t('modelLimit.description')}
            </p>
          </div>
        </div>

        {error && (
          <div className="flex items-center gap-2 rounded-lg border border-rose-200 bg-rose-50 p-3">
            <AlertCircle className="h-4 w-4 flex-shrink-0 text-rose-500" />
            <span className="text-sm text-rose-600">{error}</span>
          </div>
        )}

        {isLoading ? (
          <div className="flex items-center justify-center gap-2 py-10 text-sm text-slate-500">
            <Loader2 className="h-4 w-4 animate-spin" />
            {t('modelLimit.loading')}
          </div>
        ) : (
          <div className="space-y-3">
            <label className="block">
              <span className="text-sm font-medium text-slate-700">
                {t('modelLimit.contextLabel')}
              </span>
              <input
                type="number"
                min="1"
                step="1"
                inputMode="numeric"
                value={form.context}
                onChange={(e) => handleFieldChange('context', e.target.value)}
                placeholder="128000"
                className="mt-1 w-full rounded-lg border border-slate-200 px-3 py-2 text-sm focus:border-indigo-300 focus:outline-none focus:ring-2 focus:ring-indigo-500/20"
              />
            </label>

            <label className="block">
              <span className="text-sm font-medium text-slate-700">
                {t('modelLimit.inputLabel')}
              </span>
              <input
                type="number"
                min="1"
                step="1"
                inputMode="numeric"
                value={form.input}
                onChange={(e) => handleFieldChange('input', e.target.value)}
                placeholder="120000"
                className="mt-1 w-full rounded-lg border border-slate-200 px-3 py-2 text-sm focus:border-indigo-300 focus:outline-none focus:ring-2 focus:ring-indigo-500/20"
              />
              <span className="mt-1 block text-xs text-slate-500">
                {t('modelLimit.inputHint')}
              </span>
            </label>

            <label className="block">
              <span className="text-sm font-medium text-slate-700">
                {t('modelLimit.outputLabel')}
              </span>
              <input
                type="number"
                min="1"
                step="1"
                inputMode="numeric"
                value={form.output}
                onChange={(e) => handleFieldChange('output', e.target.value)}
                placeholder="8192"
                className="mt-1 w-full rounded-lg border border-slate-200 px-3 py-2 text-sm focus:border-indigo-300 focus:outline-none focus:ring-2 focus:ring-indigo-500/20"
              />
            </label>

            {validationError && (
              <p className="text-xs font-medium text-amber-600">{validationError}</p>
            )}
          </div>
        )}
      </div>
    </Modal>
  );
}

export default ModelLimitModal;
