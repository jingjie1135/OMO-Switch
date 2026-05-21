import { useState, useMemo } from 'react';
import { useTranslation } from 'react-i18next';
import { Plus, AlertCircle } from 'lucide-react';
import { Modal } from '../common/Modal';
import { Button } from '../common/Button';
import { addCustomModel } from '../../services/tauri';

interface AddModelModalProps {
  isOpen: boolean;
  onClose: () => void;
  currentProviderId: string;
  onModelAdded: () => void;
  existingModels: string[];
}

export function AddModelModal({
  isOpen,
  onClose,
  currentProviderId,
  onModelAdded,
  existingModels,
}: AddModelModalProps) {
  const { t } = useTranslation();
  const [isAdding, setIsAdding] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [modelId, setModelId] = useState('');

  const existingModelSet = useMemo(
    () => new Set(existingModels.map((model) => model.trim()).filter(Boolean)),
    [existingModels]
  );

  const handleClose = () => {
    setModelId('');
    setError(null);
    onClose();
  };

  const handleAddModel = async (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault();

    const trimmedModelId = modelId.trim();
    if (!trimmedModelId) {
      setError(t('customModel.modelIdRequired'));
      return;
    }
    if (existingModelSet.has(trimmedModelId)) {
      setError(t('customModel.modelAlreadyExists'));
      return;
    }

    try {
      setIsAdding(true);
      setError(null);

      await addCustomModel(currentProviderId, trimmedModelId);

      onModelAdded();

      handleClose();
    } catch (err) {
      setError(
        err instanceof Error ? err.message : t('customModel.addModelError')
      );
    } finally {
      setIsAdding(false);
    }
  };

  return (
    <Modal
      isOpen={isOpen}
      onClose={handleClose}
      title={t('customModel.addCustomModel')}
      size="md"
    >
      <form onSubmit={handleAddModel} className="space-y-4">
        <p className="text-sm text-slate-600">
          {t('customModel.manualEntryDescription')}
        </p>

        {error && (
          <div className="flex items-center gap-2 p-3 bg-rose-50 border border-rose-200 rounded-lg">
            <AlertCircle className="w-4 h-4 text-rose-500 flex-shrink-0" />
            <span className="text-sm text-rose-600">{error}</span>
          </div>
        )}

        <div className="space-y-2">
          <label
            htmlFor="custom-model-id"
            className="text-sm font-medium text-slate-700"
          >
            {t('customModel.modelIdLabel')}
          </label>
          <input
            id="custom-model-id"
            type="text"
            value={modelId}
            onChange={(event) => setModelId(event.target.value)}
            placeholder={t('customModel.modelIdPlaceholder')}
            autoFocus
            disabled={isAdding}
            className="w-full px-3 py-2 text-sm border border-slate-200 rounded-lg focus:outline-none focus:ring-2 focus:ring-emerald-500/20 focus:border-emerald-300 disabled:bg-slate-50 disabled:text-slate-400"
          />
        </div>

        <div className="flex justify-end gap-2 pt-4 border-t border-slate-100">
          <Button type="button" variant="ghost" onClick={handleClose} disabled={isAdding}>
            {t('button.cancel')}
          </Button>
          <Button type="submit" isLoading={isAdding}>
            <Plus className="w-4 h-4 mr-2" />
            {t('customModel.addModel')}
          </Button>
        </div>
      </form>
    </Modal>
  );
}

export default AddModelModal;
